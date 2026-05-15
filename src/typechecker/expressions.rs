//! Expression checking — check_function and check_expr.
#![allow(clippy::result_large_err)]

use crate::ast::expressions::StringPart;
use crate::ast::typed::*;
use crate::ast::{AstType, Expression, Param};
use crate::error::{Diagnostic, Span};

use super::closures::collect_captures;
use super::monomorphize::InferenceConflict;
use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_function(
        &mut self,
        name: &str,
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        _span: &Span,
    ) -> Result<TypedFunction, Diagnostic> {
        let ret_type = return_type
            .as_ref()
            .map(|t| self.resolve_type(t))
            .unwrap_or(Type::Void);

        self.current_return_type = Some(ret_type.clone());

        // Push function scope with params
        self.push_scope();
        let mut typed_params = Vec::new();
        for p in params {
            let ty = self.resolve_type(&p.ty);
            self.define_var(&p.name, ty.clone());
            typed_params.push(TypedParam {
                name: p.name.clone(),
                ty,
                span: p.span,
            });
        }

        let typed_body = self.check_expr(body)?;
        let body_block = match typed_body.kind {
            TypedExprKind::Block(block) => block,
            _ => TypedBlock {
                ty: typed_body.ty.clone(),
                span: typed_body.span,
                statements: Vec::new(),
                expr: Some(Box::new(typed_body)),
            },
        };

        self.pop_scope();
        self.current_return_type = None;

        if ret_type != Type::Void
            && ret_type != Type::Never
            && !self.block_satisfies_return(&body_block, &ret_type)
        {
            return Err(Diagnostic::error(
                "E3031",
                format!(
                    "function `{}` must return `{}` on all non-error paths",
                    name,
                    ret_type.display_name()
                ),
                *_span,
            ));
        }

        // Collect defers accumulated during this function's body (LIFO order)
        let mut defers: Vec<TypedExpression> = self.pending_defers.drain(..).collect();
        defers.reverse();

        Ok(TypedFunction {
            name: name.to_string(),
            params: typed_params,
            return_type: ret_type,
            body: body_block,
            defers,
            span: *_span,
        })
    }

    pub(crate) fn check_expr(&mut self, expr: &Expression) -> Result<TypedExpression, Diagnostic> {
        match expr {
            Expression::IntLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::IntLiteral(*value),
                ty: Type::I32, // default int type
                span: *span,
            }),

            Expression::FloatLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::FloatLiteral(*value),
                ty: Type::F64, // default float type
                span: *span,
            }),

            Expression::StringLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::StringLiteral(value.clone()),
                ty: Type::Str,
                span: *span,
            }),

            Expression::BoolLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::BoolLiteral(*value),
                ty: Type::Bool,
                span: *span,
            }),

            Expression::Identifier { name, span } => {
                if name == "true" {
                    return Ok(TypedExpression {
                        kind: TypedExprKind::BoolLiteral(true),
                        ty: Type::Bool,
                        span: *span,
                    });
                }
                if name == "false" {
                    return Ok(TypedExpression {
                        kind: TypedExprKind::BoolLiteral(false),
                        ty: Type::Bool,
                        span: *span,
                    });
                }
                let ty = match self.lookup_var(name) {
                    Some(t) => t,
                    None => {
                        // Not a variable — only warn if it's not a known type, enum, function, or import
                        if !self.structs.contains_key(name.as_str())
                            && !self.enums.contains_key(name.as_str())
                            && !self.functions.contains_key(name.as_str())
                            && !self.is_import(name)
                        {
                            self.diagnostics.push(Diagnostic::error(
                                "E3040",
                                format!("undefined variable `{}`", name),
                                *span,
                            ));
                        }
                        Type::Unknown
                    }
                };
                Ok(TypedExpression {
                    kind: TypedExprKind::Variable(name.clone()),
                    ty,
                    span: *span,
                })
            }

            Expression::BinaryOp {
                op,
                left,
                right,
                span,
            } => {
                let left = self.check_expr(left)?;
                let right = self.check_expr(right)?;
                let ty = self.check_binary_op(*op, &left.ty, &right.ty, span)?;
                Ok(TypedExpression {
                    kind: TypedExprKind::BinaryOp {
                        op: *op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    ty,
                    span: *span,
                })
            }

            Expression::FunctionCall {
                name,
                module,
                type_args,
                args,
                span,
            } => {
                let mut typed_args = Vec::new();
                for arg in args {
                    typed_args.push(self.check_expr(arg)?);
                }

                // Look up the function
                let full_name = if let Some(m) = module {
                    format!("{}.{}", m, name)
                } else {
                    name.clone()
                };

                let (resolved_name, ret_type) =
                    if let Some(info) = self.functions.get(&full_name).cloned() {
                        if !info.type_params.is_empty() {
                            let (subs, explicit_type_args_valid) = if type_args.is_empty() {
                                let arg_types: Vec<Type> =
                                    typed_args.iter().map(|a| a.ty.clone()).collect();
                                let (subs, conflicts) = self.infer_type_args_with_conflicts(
                                    &info.type_params,
                                    &info.params,
                                    &arg_types,
                                );
                                self.report_inference_conflicts(
                                    "function", &full_name, conflicts, *span,
                                );
                                (subs, true)
                            } else {
                                self.explicit_type_arg_substitutions(
                                    "function",
                                    &full_name,
                                    &info.type_params,
                                    type_args,
                                    *span,
                                )
                            };
                            let (ret, mangled) = if explicit_type_args_valid {
                                self.check_call_signature_with_substitutions(
                                    "function",
                                    &full_name,
                                    &info.params,
                                    &typed_args,
                                    &subs,
                                    span,
                                );
                                self.check_generic_bounds(&info.type_param_bounds, &subs, *span);
                                let ret = self.substitute_type(&info.return_type, &subs);
                                let mangled = self
                                    .specialize_generic_function(&full_name, &subs, *span)
                                    .unwrap_or_else(|| {
                                        self.generic_function_mangled_name(
                                            &full_name,
                                            &info.type_params,
                                            &subs,
                                        )
                                    });
                                (ret, mangled)
                            } else {
                                (
                                    Type::Unknown,
                                    self.generic_function_mangled_name(
                                        &full_name,
                                        &info.type_params,
                                        &subs,
                                    ),
                                )
                            };
                            (mangled, ret)
                        } else {
                            if !type_args.is_empty() {
                                self.diagnostics.push(Diagnostic::error(
                                    "E5001",
                                    format!(
                                        "non-generic function `{}` does not accept type arguments",
                                        full_name
                                    ),
                                    *span,
                                ));
                            }
                            self.check_call_signature(
                                "function",
                                &full_name,
                                &info.params,
                                &typed_args,
                                span,
                            );
                            (full_name.clone(), self.resolve_type(&info.return_type))
                        }
                    } else if name == "cast" && typed_args.len() == 1 && !type_args.is_empty() {
                        // cast(expr, Type) — handled specially
                        (full_name.clone(), self.resolve_type(&type_args[0]))
                    } else if module.is_some() {
                        // Try looking up module-qualified names in methods/functions maps
                        let mangled = if let Some(m) = module {
                            format!("{}_{}", m, name)
                        } else {
                            name.clone()
                        };
                        if let Some(info) = self.methods.get(&full_name).cloned() {
                            self.check_call_signature(
                                "method",
                                &full_name,
                                &info.params,
                                &typed_args,
                                span,
                            );
                            (full_name.clone(), self.resolve_type(&info.return_type))
                        } else if let Some(info) = self.functions.get(&mangled).cloned() {
                            self.check_call_signature(
                                "function",
                                &mangled,
                                &info.params,
                                &typed_args,
                                span,
                            );
                            (full_name.clone(), self.resolve_type(&info.return_type))
                        } else {
                            let m = module.as_deref().unwrap_or("");
                            self.diagnostics.push(Diagnostic::warning(
                                "W3041",
                                format!("unknown function `{}.{}`, assuming void return", m, name),
                                *span,
                            ));
                            (full_name.clone(), Type::Void)
                        }
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            "E3020",
                            format!("undefined function `{}`", name),
                            *span,
                        ));
                        (full_name.clone(), Type::Unknown)
                    };

                Ok(TypedExpression {
                    kind: TypedExprKind::FunctionCall {
                        function: resolved_name,
                        args: typed_args,
                    },
                    ty: ret_type,
                    span: *span,
                })
            }

            Expression::MethodCall {
                receiver,
                method,
                type_args,
                args,
                span,
            } => {
                // Check if receiver is an imported module name (like `io`)
                // In that case, this is a module-qualified call: io.println(args)
                if let Expression::Identifier {
                    name: ref recv_name,
                    ..
                } = **receiver
                {
                    if self.is_import(recv_name) {
                        let mut typed_args = Vec::new();
                        for arg in args {
                            typed_args.push(self.check_expr(arg)?);
                        }
                        let mangled = format!("{}_{}", recv_name, method);
                        // Try to look up the return type
                        let ret_type = if let Some(info) = self.functions.get(&mangled).cloned() {
                            self.check_call_signature(
                                "function",
                                &mangled,
                                &info.params,
                                &typed_args,
                                span,
                            );
                            self.resolve_type(&info.return_type)
                        } else {
                            let method_key = format!("{}.{}", recv_name, method);
                            if let Some(info) = self.methods.get(&method_key).cloned() {
                                self.check_call_signature(
                                    "method",
                                    &method_key,
                                    &info.params,
                                    &typed_args,
                                    span,
                                );
                                self.resolve_type(&info.return_type)
                            } else if self.is_root_std_runtime_call(recv_name, method) {
                                Type::Void
                            } else {
                                self.diagnostics.push(Diagnostic::error(
                                    "E3023",
                                    format!("undefined module function `{}.{}`", recv_name, method),
                                    *span,
                                ));
                                Type::Unknown
                            }
                        };
                        return Ok(TypedExpression {
                            kind: TypedExprKind::FunctionCall {
                                function: mangled,
                                args: typed_args,
                            },
                            ty: ret_type,
                            span: *span,
                        });
                    }
                }

                let typed_receiver = self.check_expr(receiver)?;

                // Build args: receiver as first arg (for methods/UFC)
                let mut typed_args = vec![typed_receiver.clone()];
                for arg in args {
                    typed_args.push(self.check_expr(arg)?);
                }

                // Try to find method on receiver type
                let type_name = match &typed_receiver.ty {
                    Type::Named(n) | Type::Struct { name: n, .. } | Type::Enum { name: n, .. } => {
                        n.clone()
                    }
                    _ => String::new(),
                };
                let method_key = format!("{}.{}", type_name, method);

                if let Some(info) = self.methods.get(&method_key).cloned() {
                    // Found as a type method — handle generics
                    let (resolved_method, ret_type) = if !info.type_params.is_empty() {
                        let (subs, explicit_type_args_valid) = if type_args.is_empty() {
                            let arg_types: Vec<Type> =
                                typed_args.iter().map(|a| a.ty.clone()).collect();
                            let (subs, conflicts) = self.infer_method_type_args(
                                &method_key,
                                &info.type_params,
                                &info.params,
                                &arg_types,
                            );
                            self.report_inference_conflicts(
                                "method",
                                &method_key,
                                conflicts,
                                *span,
                            );
                            (subs, true)
                        } else {
                            self.explicit_type_arg_substitutions(
                                "method",
                                &method_key,
                                &info.type_params,
                                type_args,
                                *span,
                            )
                        };
                        let (ret, mangled) = if explicit_type_args_valid {
                            let saved_self_type = self.current_self_type.clone();
                            self.current_self_type =
                                self.generic_method_self_type(&method_key, &subs);
                            self.check_call_signature_with_substitutions(
                                "method",
                                &method_key,
                                &info.params,
                                &typed_args,
                                &subs,
                                span,
                            );
                            self.check_generic_bounds(&info.type_param_bounds, &subs, *span);
                            let ret = self.substitute_type(&info.return_type, &subs);
                            let mangled = self
                                .specialize_generic_method(&method_key, &subs, *span)
                                .unwrap_or_else(|| {
                                    self.generic_function_mangled_name(
                                        &method_key,
                                        &info.type_params,
                                        &subs,
                                    )
                                });
                            self.current_self_type = saved_self_type;
                            (ret, mangled)
                        } else {
                            (
                                Type::Unknown,
                                self.generic_function_mangled_name(
                                    &method_key,
                                    &info.type_params,
                                    &subs,
                                ),
                            )
                        };
                        (mangled, ret)
                    } else {
                        if !type_args.is_empty() {
                            self.diagnostics.push(Diagnostic::error(
                                "E5001",
                                format!(
                                    "non-generic method `{}` does not accept type arguments",
                                    method_key
                                ),
                                *span,
                            ));
                        }
                        self.check_call_signature(
                            "method",
                            &method_key,
                            &info.params,
                            &typed_args,
                            span,
                        );
                        (
                            format!("{}_{}", type_name, method),
                            self.resolve_type(&info.return_type),
                        )
                    };
                    Ok(TypedExpression {
                        kind: TypedExprKind::FunctionCall {
                            function: resolved_method,
                            args: typed_args,
                        },
                        ty: ret_type,
                        span: *span,
                    })
                } else if let Some(generic_base) = self.generic_base_type_name(&type_name) {
                    let generic_method_key = format!("{}.{}", generic_base, method);
                    if let Some(info) = self.methods.get(&generic_method_key).cloned() {
                        if !info.type_params.is_empty() {
                            let (subs, explicit_type_args_valid) = if type_args.is_empty() {
                                let arg_types: Vec<Type> =
                                    typed_args.iter().map(|a| a.ty.clone()).collect();
                                let (subs, conflicts) = self.infer_method_type_args(
                                    &generic_method_key,
                                    &info.type_params,
                                    &info.params,
                                    &arg_types,
                                );
                                self.report_inference_conflicts(
                                    "method",
                                    &generic_method_key,
                                    conflicts,
                                    *span,
                                );
                                (subs, true)
                            } else {
                                self.explicit_type_arg_substitutions(
                                    "method",
                                    &generic_method_key,
                                    &info.type_params,
                                    type_args,
                                    *span,
                                )
                            };
                            let (ret_type, mangled) = if explicit_type_args_valid {
                                let saved_self_type = self.current_self_type.clone();
                                self.current_self_type =
                                    self.generic_method_self_type(&generic_method_key, &subs);
                                self.check_call_signature_with_substitutions(
                                    "method",
                                    &generic_method_key,
                                    &info.params,
                                    &typed_args,
                                    &subs,
                                    span,
                                );
                                self.check_generic_bounds(&info.type_param_bounds, &subs, *span);
                                let ret_type = self.substitute_type(&info.return_type, &subs);
                                let mangled = self
                                    .specialize_generic_method(&generic_method_key, &subs, *span)
                                    .unwrap_or_else(|| {
                                        self.generic_function_mangled_name(
                                            &generic_method_key,
                                            &info.type_params,
                                            &subs,
                                        )
                                    });
                                self.current_self_type = saved_self_type;
                                (ret_type, mangled)
                            } else {
                                (
                                    Type::Unknown,
                                    self.generic_function_mangled_name(
                                        &generic_method_key,
                                        &info.type_params,
                                        &subs,
                                    ),
                                )
                            };
                            Ok(TypedExpression {
                                kind: TypedExprKind::FunctionCall {
                                    function: mangled,
                                    args: typed_args,
                                },
                                ty: ret_type,
                                span: *span,
                            })
                        } else {
                            if !type_args.is_empty() {
                                self.diagnostics.push(Diagnostic::error(
                                    "E5001",
                                    format!(
                                        "non-generic method `{}` does not accept type arguments",
                                        generic_method_key
                                    ),
                                    *span,
                                ));
                            }
                            self.check_call_signature(
                                "method",
                                &generic_method_key,
                                &info.params,
                                &typed_args,
                                span,
                            );
                            Ok(TypedExpression {
                                kind: TypedExprKind::FunctionCall {
                                    function: format!("{}_{}", generic_base, method),
                                    args: typed_args,
                                },
                                ty: self.resolve_type(&info.return_type),
                                span: *span,
                            })
                        }
                    } else {
                        self.unknown_method_expr(&type_name, method, typed_args, *span)
                    }
                } else if let Some(info) = self.functions.get(method).cloned() {
                    // UFC: x.f(args) -> f(x, args)
                    let (resolved_function, ret_type) = if !info.type_params.is_empty() {
                        let (subs, explicit_type_args_valid) = if type_args.is_empty() {
                            let arg_types: Vec<Type> =
                                typed_args.iter().map(|a| a.ty.clone()).collect();
                            let (subs, conflicts) = self.infer_type_args_with_conflicts(
                                &info.type_params,
                                &info.params,
                                &arg_types,
                            );
                            self.report_inference_conflicts("function", method, conflicts, *span);
                            (subs, true)
                        } else {
                            self.explicit_type_arg_substitutions(
                                "function",
                                method,
                                &info.type_params,
                                type_args,
                                *span,
                            )
                        };
                        let (ret, mangled) = if explicit_type_args_valid {
                            self.check_call_signature_with_substitutions(
                                "function",
                                method,
                                &info.params,
                                &typed_args,
                                &subs,
                                span,
                            );
                            self.check_generic_bounds(&info.type_param_bounds, &subs, *span);
                            let ret = self.substitute_type(&info.return_type, &subs);
                            let mangled = self
                                .specialize_generic_function(method, &subs, *span)
                                .unwrap_or_else(|| {
                                    self.generic_function_mangled_name(
                                        method,
                                        &info.type_params,
                                        &subs,
                                    )
                                });
                            (ret, mangled)
                        } else {
                            (
                                Type::Unknown,
                                self.generic_function_mangled_name(
                                    method,
                                    &info.type_params,
                                    &subs,
                                ),
                            )
                        };
                        (mangled, ret)
                    } else {
                        if !type_args.is_empty() {
                            self.diagnostics.push(Diagnostic::error(
                                "E5001",
                                format!(
                                    "non-generic function `{}` does not accept type arguments",
                                    method
                                ),
                                *span,
                            ));
                        }
                        self.check_call_signature(
                            "function",
                            method,
                            &info.params,
                            &typed_args,
                            span,
                        );
                        (method.clone(), self.resolve_type(&info.return_type))
                    };
                    Ok(TypedExpression {
                        kind: TypedExprKind::FunctionCall {
                            function: resolved_function,
                            args: typed_args,
                        },
                        ty: ret_type,
                        span: *span,
                    })
                } else {
                    self.unknown_method_expr(&type_name, method, typed_args, *span)
                }
            }

            Expression::MemberAccess {
                object,
                field,
                span,
            } => {
                let typed_obj = self.check_expr(object)?;
                if field == "value"
                    && matches!(
                        typed_obj.ty,
                        Type::Ptr(_) | Type::MutPtr(_) | Type::RawPtr(_)
                    )
                {
                    let inner_ty = match &typed_obj.ty {
                        Type::Ptr(inner) | Type::MutPtr(inner) | Type::RawPtr(inner) => {
                            *inner.clone()
                        }
                        _ => Type::Unknown,
                    };
                    return Ok(TypedExpression {
                        kind: TypedExprKind::Deref(Box::new(typed_obj)),
                        ty: inner_ty,
                        span: *span,
                    });
                }

                match field.as_str() {
                    // x.addr → immutable pointer: Ptr<typeof(x)>
                    "addr" => {
                        let ptr_ty = Type::Ptr(Box::new(typed_obj.ty.clone()));
                        Ok(TypedExpression {
                            kind: TypedExprKind::Ref(Box::new(typed_obj)),
                            ty: ptr_ty,
                            span: *span,
                        })
                    }
                    // x.ref → mutable pointer: MutPtr<typeof(x)>
                    "ref" => {
                        let ptr_ty = Type::MutPtr(Box::new(typed_obj.ty.clone()));
                        Ok(TypedExpression {
                            kind: TypedExprKind::MutRef(Box::new(typed_obj)),
                            ty: ptr_ty,
                            span: *span,
                        })
                    }
                    _ => {
                        let field_type = self.lookup_field_type(&typed_obj.ty, field);
                        if field_type == Type::Unknown {
                            if let Some(type_name) = self.field_access_type_name(&typed_obj.ty) {
                                self.diagnostics.push(Diagnostic::error(
                                    "E3052",
                                    format!("type `{}` has no field `{}`", type_name, field),
                                    *span,
                                ));
                            }
                        }
                        Ok(TypedExpression {
                            kind: TypedExprKind::FieldAccess {
                                object: Box::new(typed_obj),
                                field: field.clone(),
                            },
                            ty: field_type,
                            span: *span,
                        })
                    }
                }
            }

            Expression::StructLiteral {
                name,
                type_args,
                fields,
                span,
            } => {
                let struct_info = self.structs.get(name).cloned();
                let (type_name, ty, field_defs) = if type_args.is_empty() {
                    let ty = self.resolve_type(&AstType::Named(name.clone()));
                    let field_defs = struct_info
                        .as_ref()
                        .map(|info| {
                            info.fields
                                .iter()
                                .map(|(field_name, field_type)| {
                                    (field_name.clone(), self.resolve_type(field_type))
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    (name.clone(), ty, field_defs)
                } else {
                    let type_name = self.mangle_generic_type_name(name, type_args);
                    let ty = self.resolve_type(&AstType::Generic {
                        name: name.clone(),
                        type_args: type_args.clone(),
                    });
                    let field_defs = self.specialize_generic_struct(name, type_args, *span);
                    (type_name, ty, field_defs)
                };
                let mut typed_fields = Vec::new();
                let mut provided = std::collections::HashSet::new();

                for (field_name, field_expr) in fields {
                    let typed = self.check_expr(field_expr)?;

                    if !provided.insert(field_name.as_str()) {
                        self.diagnostics.push(Diagnostic::error(
                            "E3034",
                            format!("duplicate field `{}` for struct `{}`", field_name, name),
                            typed.span,
                        ));
                    }

                    if let Some(expected) = field_defs.get(field_name) {
                        if *expected != Type::Unknown
                            && typed.ty != Type::Unknown
                            && !self.types_compatible(expected, &typed.ty)
                        {
                            self.diagnostics.push(Diagnostic::error(
                                "E3036",
                                format!(
                                    "field `{}` for struct `{}` expects `{}`, found `{}`",
                                    field_name,
                                    name,
                                    expected.display_name(),
                                    typed.ty.display_name()
                                ),
                                typed.span,
                            ));
                        }
                    } else if struct_info.is_some() {
                        self.diagnostics.push(Diagnostic::error(
                            "E3035",
                            format!("unknown field `{}` for struct `{}`", field_name, name),
                            typed.span,
                        ));
                    }

                    typed_fields.push((field_name.clone(), typed));
                }

                if let Some(info) = &struct_info {
                    for (field_name, _) in &info.fields {
                        if !provided.contains(field_name.as_str()) {
                            self.diagnostics.push(Diagnostic::error(
                                "E3037",
                                format!("missing field `{}` for struct `{}`", field_name, name),
                                *span,
                            ));
                        }
                    }
                }

                Ok(TypedExpression {
                    kind: TypedExprKind::StructLiteral {
                        type_name,
                        fields: typed_fields,
                    },
                    ty,
                    span: *span,
                })
            }

            Expression::EnumVariant {
                enum_name,
                type_args,
                variant,
                payload,
                span,
            } => {
                let typed_payload = match payload {
                    Some(p) => Some(Box::new(self.check_expr(p)?)),
                    None => None,
                };
                let (type_name, ty, variant_defs) = if type_args.is_empty() {
                    let ty = self.resolve_type(&AstType::Named(enum_name.clone()));
                    let variant_defs = self
                        .enums
                        .get(enum_name)
                        .map(|info| {
                            info.variants
                                .iter()
                                .map(|(variant_name, payload)| {
                                    (
                                        variant_name.clone(),
                                        payload.as_ref().map(|ty| self.resolve_type(ty)),
                                    )
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    (enum_name.clone(), ty, variant_defs)
                } else {
                    let type_name = self.mangle_generic_type_name(enum_name, type_args);
                    let ty = self.resolve_type(&AstType::Generic {
                        name: enum_name.clone(),
                        type_args: type_args.clone(),
                    });
                    let variant_defs = self.specialize_generic_enum(enum_name, type_args, *span);
                    (type_name, ty, variant_defs)
                };
                if self.enums.contains_key(enum_name) {
                    match variant_defs.get(variant) {
                        Some(expected_payload) => match (expected_payload, &typed_payload) {
                            (Some(expected_ast), Some(actual)) => {
                                let expected = expected_ast.clone();
                                if expected != Type::Unknown
                                    && actual.ty != Type::Unknown
                                    && !self.types_compatible(&expected, &actual.ty)
                                {
                                    self.diagnostics.push(Diagnostic::error(
                                        "E3062",
                                        format!(
                                            "payload for enum variant `{}.{}` expects `{}`, found `{}`",
                                            enum_name,
                                            variant,
                                            expected.display_name(),
                                            actual.ty.display_name()
                                        ),
                                        actual.span,
                                    ));
                                }
                            }
                            (Some(_), None) => {
                                self.diagnostics.push(Diagnostic::error(
                                    "E3061",
                                    format!(
                                        "enum variant `{}.{}` requires a payload",
                                        enum_name, variant
                                    ),
                                    *span,
                                ));
                            }
                            (None, Some(actual)) => {
                                self.diagnostics.push(Diagnostic::error(
                                    "E3063",
                                    format!(
                                        "enum variant `{}.{}` does not accept a payload",
                                        enum_name, variant
                                    ),
                                    actual.span,
                                ));
                            }
                            (None, None) => {}
                        },
                        None => {
                            self.diagnostics.push(Diagnostic::error(
                                "E3060",
                                format!("enum `{}` has no variant `{}`", enum_name, variant),
                                *span,
                            ));
                        }
                    }
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        "E3064",
                        format!("undefined enum `{}`", enum_name),
                        *span,
                    ));
                }
                Ok(TypedExpression {
                    kind: TypedExprKind::EnumVariant {
                        type_name,
                        variant: variant.clone(),
                        payload: typed_payload,
                    },
                    ty,
                    span: *span,
                })
            }

            Expression::ArrayLiteral { elements, span } => {
                let mut typed_elems = Vec::new();
                let mut elem_type = Type::Unknown;
                for elem in elements {
                    let typed = self.check_expr(elem)?;
                    if elem_type == Type::Unknown {
                        elem_type = typed.ty.clone();
                    }
                    typed_elems.push(typed);
                }
                if elements.is_empty() {
                    self.diagnostics.push(Diagnostic::warning(
                        "W3045",
                        "cannot infer element type for empty array".to_string(),
                        *span,
                    ));
                }
                Ok(TypedExpression {
                    kind: TypedExprKind::ArrayLiteral {
                        elements: typed_elems,
                    },
                    ty: Type::Array {
                        elem: Box::new(elem_type),
                        size: Some(elements.len()),
                    },
                    span: *span,
                })
            }

            Expression::Block {
                statements,
                expr,
                span,
            } => {
                self.push_scope();
                let mut typed_stmts = Vec::new();
                for stmt in statements {
                    typed_stmts.push(self.check_statement(stmt)?);
                }
                let typed_expr = match expr {
                    Some(e) => Some(Box::new(self.check_expr(e)?)),
                    None => None,
                };
                let ty = typed_expr
                    .as_ref()
                    .map(|e| e.ty.clone())
                    .unwrap_or(Type::Void);
                self.pop_scope();
                Ok(TypedExpression {
                    kind: TypedExprKind::Block(TypedBlock {
                        statements: typed_stmts,
                        expr: typed_expr,
                        ty: ty.clone(),
                        span: *span,
                    }),
                    ty,
                    span: *span,
                })
            }

            Expression::Return { value, span } => {
                let typed_val = match value {
                    Some(v) => Some(Box::new(self.check_expr(v)?)),
                    None => None,
                };
                // Check return type compatibility
                if let Some(ref expected) = self.current_return_type {
                    let actual = typed_val.as_ref().map(|v| &v.ty).unwrap_or(&Type::Void);
                    if !self.types_compatible(expected, actual) {
                        self.diagnostics.push(Diagnostic::error(
                            "E3030",
                            format!(
                                "return type mismatch: expected `{}`, found `{}`",
                                expected.display_name(),
                                actual.display_name()
                            ),
                            *span,
                        ));
                    }
                }
                Ok(TypedExpression {
                    kind: TypedExprKind::Return(typed_val),
                    ty: Type::Never,
                    span: *span,
                })
            }

            Expression::Break { span } => Ok(TypedExpression {
                kind: TypedExprKind::Break,
                ty: Type::Never,
                span: *span,
            }),

            Expression::Continue { span } => Ok(TypedExpression {
                kind: TypedExprKind::Continue,
                ty: Type::Never,
                span: *span,
            }),

            Expression::Match {
                scrutinee,
                arms,
                span,
            } => {
                let typed_scrutinee = self.check_expr(scrutinee)?;
                let mut typed_arms = Vec::new();
                let mut result_type = Type::Void;
                let mut saw_value_arm = false;
                let mut saw_never_arm = false;

                for arm in arms {
                    self.push_scope();
                    self.bind_pattern(&arm.pattern, &typed_scrutinee.ty);
                    let typed_body = self.check_expr(&arm.body)?;
                    if typed_body.ty == Type::Never {
                        saw_never_arm = true;
                    } else if !saw_value_arm && typed_body.ty != Type::Void {
                        result_type = typed_body.ty.clone();
                        saw_value_arm = true;
                    }
                    let pattern = self.lower_pattern(&arm.pattern, &typed_scrutinee.ty);
                    self.pop_scope();
                    typed_arms.push(TypedMatchArm {
                        pattern,
                        body: TypedBlock {
                            ty: typed_body.ty.clone(),
                            span: typed_body.span,
                            statements: Vec::new(),
                            expr: Some(Box::new(typed_body)),
                        },
                        span: arm.span,
                    });
                }
                if !saw_value_arm && saw_never_arm {
                    result_type = Type::Never;
                }

                // Determine match kind
                let kind = self.determine_match_kind(&typed_scrutinee.ty, arms);
                if matches!(kind, MatchKind::EnumMatch) {
                    self.check_enum_match_patterns(&typed_scrutinee.ty, arms);
                    self.check_match_exhaustiveness(&typed_scrutinee.ty, arms, *span);
                } else if matches!(kind, MatchKind::Conditional | MatchKind::ConditionalElse) {
                    self.check_bool_match_patterns(arms, result_type != Type::Void, *span);
                }

                Ok(TypedExpression {
                    kind: TypedExprKind::Match {
                        scrutinee: Box::new(typed_scrutinee),
                        arms: typed_arms,
                        kind,
                    },
                    ty: result_type,
                    span: *span,
                })
            }

            Expression::If {
                condition,
                then_body,
                else_body,
                span,
            } => {
                let typed_cond = self.check_expr(condition)?;
                let typed_then = self.check_expr(then_body)?;
                let typed_else = match else_body {
                    Some(e) => Some(Box::new(self.check_expr(e)?)),
                    None => None,
                };

                let ty = typed_then.ty.clone();
                let then_block = TypedBlock {
                    ty: typed_then.ty.clone(),
                    span: typed_then.span,
                    statements: Vec::new(),
                    expr: Some(Box::new(typed_then)),
                };
                let else_arm = typed_else.map(|e| TypedMatchArm {
                    pattern: TypedPattern::Bool(false),
                    body: TypedBlock {
                        ty: e.ty.clone(),
                        span: e.span,
                        statements: Vec::new(),
                        expr: Some(e),
                    },
                    span: *span,
                });

                let mut arms = vec![TypedMatchArm {
                    pattern: TypedPattern::Bool(true),
                    body: then_block,
                    span: *span,
                }];
                if let Some(ea) = else_arm {
                    arms.push(ea);
                }

                Ok(TypedExpression {
                    kind: TypedExprKind::Match {
                        scrutinee: Box::new(typed_cond),
                        arms,
                        kind: if else_body.is_some() {
                            MatchKind::ConditionalElse
                        } else {
                            MatchKind::Conditional
                        },
                    },
                    ty,
                    span: *span,
                })
            }

            Expression::WhileLoop {
                condition,
                body,
                span,
            } => {
                let typed_cond = self.check_expr(condition)?;
                let typed_body = self.check_expr(body)?;
                let body_block = TypedBlock {
                    ty: typed_body.ty.clone(),
                    span: typed_body.span,
                    statements: Vec::new(),
                    expr: Some(Box::new(typed_body)),
                };

                Ok(TypedExpression {
                    kind: TypedExprKind::Match {
                        scrutinee: Box::new(typed_cond),
                        arms: vec![TypedMatchArm {
                            pattern: TypedPattern::Bool(true),
                            body: body_block,
                            span: *span,
                        }],
                        kind: MatchKind::WhileLoop,
                    },
                    ty: Type::Void,
                    span: *span,
                })
            }

            Expression::Loop { body, span } => {
                let typed_body = self.check_expr(body)?;
                // Loop desugars to WhileLoop with `true` scrutinee
                let body_block = TypedBlock {
                    ty: typed_body.ty.clone(),
                    span: typed_body.span,
                    statements: Vec::new(),
                    expr: Some(Box::new(typed_body)),
                };

                Ok(TypedExpression {
                    kind: TypedExprKind::Match {
                        scrutinee: Box::new(TypedExpression {
                            kind: TypedExprKind::BoolLiteral(true),
                            ty: Type::Bool,
                            span: *span,
                        }),
                        arms: vec![TypedMatchArm {
                            pattern: TypedPattern::Bool(true),
                            body: body_block,
                            span: *span,
                        }],
                        kind: MatchKind::WhileLoop,
                    },
                    ty: Type::Void,
                    span: *span,
                })
            }

            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                let typed_expr = self.check_expr(expr)?;
                let from_type = typed_expr.ty.clone();
                let to_type = self.resolve_type(target_type);
                Ok(TypedExpression {
                    kind: TypedExprKind::Cast {
                        expr: Box::new(typed_expr),
                        from_type,
                        to_type: to_type.clone(),
                    },
                    ty: to_type,
                    span: *span,
                })
            }

            Expression::StringInterpolation { parts, span } => {
                let mut typed_parts = Vec::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => {
                            typed_parts.push(TypedStringPart::Literal(s.clone()));
                        }
                        StringPart::Expr(e) => {
                            let typed = self.check_expr(e)?;
                            typed_parts.push(TypedStringPart::Expr(typed));
                        }
                    }
                }
                Ok(TypedExpression {
                    kind: TypedExprKind::StringInterpolation { parts: typed_parts },
                    ty: Type::Str,
                    span: *span,
                })
            }

            Expression::Defer { expr, span } => {
                // Type-check the deferred expression and collect it
                let typed_expr = self.check_expr(expr)?;
                self.pending_defers.push(typed_expr);
                // Defer itself doesn't produce a value
                Ok(TypedExpression {
                    kind: TypedExprKind::Block(TypedBlock {
                        statements: Vec::new(),
                        expr: None,
                        ty: Type::Void,
                        span: *span,
                    }),
                    ty: Type::Void,
                    span: *span,
                })
            }

            Expression::IndexAccess {
                object,
                index,
                span,
            } => {
                let typed_obj = self.check_expr(object)?;
                let typed_idx = self.check_expr(index)?;
                let elem_ty = match &typed_obj.ty {
                    Type::Array { elem, .. } | Type::Slice(elem) => *elem.clone(),
                    _ => {
                        self.diagnostics.push(Diagnostic::error(
                            "E3051",
                            format!(
                                "cannot index into non-array type `{}`",
                                typed_obj.ty.display_name()
                            ),
                            *span,
                        ));
                        Type::Unknown
                    }
                };
                Ok(TypedExpression {
                    kind: TypedExprKind::IndexAccess {
                        object: Box::new(typed_obj),
                        index: Box::new(typed_idx),
                    },
                    ty: elem_ty,
                    span: *span,
                })
            }

            Expression::Closure {
                params,
                return_type: _return_type,
                body,
                span,
            } => {
                // Collect variables visible before entering closure scope
                let outer_vars: std::collections::HashMap<String, Type> = self
                    .scopes
                    .iter()
                    .flat_map(|s| s.vars.iter())
                    .map(|(k, v)| (k.clone(), v.ty.clone()))
                    .collect();

                self.push_scope();
                let mut param_types = Vec::new();
                let mut param_names: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                for p in params {
                    let ty = self.resolve_type(&p.ty);
                    self.define_var(&p.name, ty.clone());
                    param_types.push(ty);
                    param_names.insert(p.name.clone());
                }
                let typed_body = self.check_expr(body)?;
                self.pop_scope();

                let ret_type = if let Some(rt) = _return_type {
                    self.resolve_type(rt)
                } else {
                    typed_body.ty.clone()
                };

                // Capture analysis: find variables in body that come from outer scopes
                let mut captures = Vec::new();
                let mut seen = std::collections::HashSet::new();
                collect_captures(
                    &typed_body,
                    &param_names,
                    &outer_vars,
                    &mut captures,
                    &mut seen,
                );

                let fn_name = format!("__closure_{}_{}", span.start, span.end);
                let env_type = if captures.is_empty() {
                    String::new()
                } else {
                    format!("__env_{}_{}", span.start, span.end)
                };

                let fn_type = Type::Function {
                    params: param_types,
                    ret: Box::new(ret_type),
                };

                Ok(TypedExpression {
                    kind: TypedExprKind::Closure {
                        fn_name,
                        env_type,
                        captures,
                    },
                    ty: fn_type,
                    span: *span,
                })
            }

            Expression::UnaryOp { op, operand, span } => {
                let typed = self.check_expr(operand)?;
                let ty = typed.ty.clone();
                Ok(TypedExpression {
                    kind: TypedExprKind::UnaryOp {
                        op: *op,
                        operand: Box::new(typed),
                    },
                    ty,
                    span: *span,
                })
            }

            // TODO: implement Range type
            Expression::Range {
                start, end, span, ..
            } => {
                let _typed_start = self.check_expr(start)?;
                let _typed_end = self.check_expr(end)?;
                Ok(TypedExpression {
                    kind: TypedExprKind::Error,
                    ty: Type::Unknown,
                    span: *span,
                })
            }

            // TODO: implement char literal type
            Expression::Error { span } | Expression::CharLiteral { span, .. } => {
                Ok(TypedExpression {
                    kind: TypedExprKind::Error,
                    ty: Type::Unknown,
                    span: *span,
                })
            }
        }
    }

    fn check_call_signature(
        &mut self,
        kind: &str,
        callee: &str,
        params: &[(String, AstType)],
        args: &[TypedExpression],
        span: &Span,
    ) {
        if params.len() != args.len() {
            self.diagnostics.push(Diagnostic::error(
                "E3021",
                format!(
                    "{} `{}` expects {} arguments, found {}",
                    kind,
                    callee,
                    params.len(),
                    args.len()
                ),
                *span,
            ));
            return;
        }

        for (idx, ((_, expected), actual)) in params.iter().zip(args.iter()).enumerate() {
            let expected = self.resolve_type(expected);
            if expected == Type::Unknown || actual.ty == Type::Unknown {
                continue;
            }

            if !self.types_compatible(&expected, &actual.ty) {
                self.diagnostics.push(Diagnostic::error(
                    "E3022",
                    format!(
                        "argument {} for `{}` expects `{}`, found `{}`",
                        idx + 1,
                        callee,
                        expected.display_name(),
                        actual.ty.display_name()
                    ),
                    actual.span,
                ));
            }
        }
    }

    fn check_call_signature_with_substitutions(
        &mut self,
        kind: &str,
        callee: &str,
        params: &[(String, AstType)],
        args: &[TypedExpression],
        substitutions: &std::collections::HashMap<String, Type>,
        span: &Span,
    ) {
        if params.len() != args.len() {
            self.diagnostics.push(Diagnostic::error(
                "E3021",
                format!(
                    "{} `{}` expects {} arguments, found {}",
                    kind,
                    callee,
                    params.len(),
                    args.len()
                ),
                *span,
            ));
            return;
        }

        for (idx, ((_, expected), actual)) in params.iter().zip(args.iter()).enumerate() {
            let expected = self.substitute_type(expected, substitutions);
            if expected == Type::Unknown || actual.ty == Type::Unknown {
                continue;
            }

            if !self.types_compatible(&expected, &actual.ty) {
                self.diagnostics.push(Diagnostic::error(
                    "E3022",
                    format!(
                        "argument {} for `{}` expects `{}`, found `{}`",
                        idx + 1,
                        callee,
                        expected.display_name(),
                        actual.ty.display_name()
                    ),
                    actual.span,
                ));
            }
        }
    }

    fn report_inference_conflicts(
        &mut self,
        kind: &str,
        callee: &str,
        conflicts: Vec<InferenceConflict>,
        span: Span,
    ) {
        for conflict in conflicts {
            self.diagnostics.push(Diagnostic::error(
                "E5000",
                format!(
                    "conflicting inferred type argument `{}` for generic {} `{}`: inferred `{}` and `{}`",
                    conflict.param,
                    kind,
                    callee,
                    conflict.inferred.display_name(),
                    conflict.actual.display_name()
                ),
                span,
            ));
        }
    }

    fn explicit_type_arg_substitutions(
        &mut self,
        kind: &str,
        callee: &str,
        type_params: &[String],
        type_args: &[AstType],
        span: Span,
    ) -> (std::collections::HashMap<String, Type>, bool) {
        let arity_valid = Self::explicit_type_args_valid(type_args, type_params);
        let diagnostic_count = self.diagnostics.len();
        let substitutions =
            self.type_param_substitutions(type_params, type_args, kind, callee, span);
        let resolved_without_errors = self.diagnostics.len() == diagnostic_count;
        let annotations_valid = type_args
            .iter()
            .all(|type_arg| self.generic_type_annotation_arities_valid(type_arg));
        (
            substitutions,
            arity_valid && annotations_valid && resolved_without_errors,
        )
    }

    fn explicit_type_args_valid(type_args: &[AstType], type_params: &[String]) -> bool {
        type_args.is_empty() || type_args.len() == type_params.len()
    }

    fn generic_type_annotation_arities_valid(&self, ast_type: &AstType) -> bool {
        match ast_type {
            AstType::Generic { name, type_args } => {
                let own_arity_valid = self
                    .structs
                    .get(name)
                    .map(|info| info.type_params.len())
                    .or_else(|| self.enums.get(name).map(|info| info.type_params.len()))
                    .is_none_or(|expected| expected == type_args.len());
                own_arity_valid
                    && type_args
                        .iter()
                        .all(|type_arg| self.generic_type_annotation_arities_valid(type_arg))
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => self.generic_type_annotation_arities_valid(inner),
            AstType::Array { elem, .. } => self.generic_type_annotation_arities_valid(elem),
            AstType::Function { params, ret } => {
                params
                    .iter()
                    .all(|param| self.generic_type_annotation_arities_valid(param))
                    && self.generic_type_annotation_arities_valid(ret)
            }
            _ => true,
        }
    }

    fn field_access_type_name(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Struct { name, .. } => Some(name.clone()),
            Type::Named(name) if self.structs.contains_key(name) => Some(name.clone()),
            Type::Ptr(inner) | Type::MutPtr(inner) => self.field_access_type_name(inner),
            _ => None,
        }
    }

    fn generic_base_type_name(&self, concrete_name: &str) -> Option<String> {
        self.structs
            .values()
            .filter(|info| !info.type_params.is_empty())
            .find(|info| concrete_name.starts_with(&format!("{}_", info.name)))
            .map(|info| info.name.clone())
            .or_else(|| {
                self.enums
                    .values()
                    .filter(|info| !info.type_params.is_empty())
                    .find(|info| concrete_name.starts_with(&format!("{}_", info.name)))
                    .map(|info| info.name.clone())
            })
    }

    fn unknown_method_expr(
        &mut self,
        type_name: &str,
        method: &str,
        typed_args: Vec<TypedExpression>,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        if !type_name.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "E3043",
                format!("type `{}` has no method `{}`", type_name, method),
                span,
            ));
        }
        Ok(TypedExpression {
            kind: TypedExprKind::FunctionCall {
                function: format!("{}_{}", type_name, method),
                args: typed_args,
            },
            ty: Type::Unknown,
            span,
        })
    }

    fn is_root_std_runtime_call(&self, module: &str, function: &str) -> bool {
        self.is_root_std_import(module)
            && matches!((module, function), ("io", "print") | ("io", "println"))
    }

    fn block_satisfies_return(&self, block: &TypedBlock, ret_type: &Type) -> bool {
        if block.ty != Type::Void && self.types_compatible(ret_type, &block.ty) {
            return true;
        }

        self.block_definitely_returns(block)
    }

    fn block_definitely_returns(&self, block: &TypedBlock) -> bool {
        block
            .expr
            .as_ref()
            .is_some_and(|expr| self.expr_definitely_returns(expr))
            || block.statements.iter().any(|stmt| match &stmt.kind {
                TypedStatementKind::Expression(expr) => self.expr_definitely_returns(expr),
                TypedStatementKind::VarDecl { .. } => false,
            })
    }

    fn expr_definitely_returns(&self, expr: &TypedExpression) -> bool {
        match &expr.kind {
            TypedExprKind::Return(_) => true,
            TypedExprKind::Block(block) => self.block_definitely_returns(block),
            TypedExprKind::Match { arms, .. } => {
                !arms.is_empty()
                    && arms
                        .iter()
                        .all(|arm| self.block_definitely_returns(&arm.body))
            }
            _ => false,
        }
    }
}
