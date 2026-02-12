//! Expression checking — check_function and check_expr.
#![allow(clippy::result_large_err)]

use crate::ast::expressions::StringPart;
use crate::ast::typed::*;
use crate::ast::{AstType, Expression, Param};
use crate::error::{Diagnostic, Span};

use super::closures::collect_captures;
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
                            // Generic function: infer type args and substitute
                            let arg_types: Vec<Type> =
                                typed_args.iter().map(|a| a.ty.clone()).collect();
                            let subs =
                                self.infer_type_args(&info.type_params, &info.params, &arg_types);
                            let ret = self.substitute_type(&info.return_type, &subs);
                            // Mangle name with concrete types
                            let suffix: Vec<String> = info
                                .type_params
                                .iter()
                                .filter_map(|tp| subs.get(tp).map(|t| t.display_name()))
                                .collect();
                            let mangled = if suffix.is_empty() {
                                full_name.clone()
                            } else {
                                format!("{}_{}", full_name, suffix.join("_"))
                            };
                            (mangled, ret)
                        } else {
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
                            (full_name.clone(), self.resolve_type(&info.return_type))
                        } else if let Some(info) = self.functions.get(&mangled).cloned() {
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
                type_args: _,
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
                        let ret_type = if let Some(info) = self.functions.get(&mangled) {
                            self.resolve_type(&info.return_type)
                        } else {
                            let method_key = format!("{}.{}", recv_name, method);
                            if let Some(info) = self.methods.get(&method_key) {
                                self.resolve_type(&info.return_type)
                            } else {
                                self.diagnostics.push(Diagnostic::warning(
                                    "W3042",
                                    format!("unknown method `{}`, assuming void return", mangled),
                                    *span,
                                ));
                                Type::Void
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
                    let ret_type = if !info.type_params.is_empty() {
                        let arg_types: Vec<Type> =
                            typed_args.iter().map(|a| a.ty.clone()).collect();
                        let subs =
                            self.infer_type_args(&info.type_params, &info.params, &arg_types);
                        self.substitute_type(&info.return_type, &subs)
                    } else {
                        self.resolve_type(&info.return_type)
                    };
                    let mangled = format!("{}_{}", type_name, method);
                    Ok(TypedExpression {
                        kind: TypedExprKind::FunctionCall {
                            function: mangled,
                            args: typed_args,
                        },
                        ty: ret_type,
                        span: *span,
                    })
                } else if let Some(info) = self.functions.get(method).cloned() {
                    // UFC: x.f(args) → f(x, args) — handle generics
                    let ret_type = if !info.type_params.is_empty() {
                        let arg_types: Vec<Type> =
                            typed_args.iter().map(|a| a.ty.clone()).collect();
                        let subs =
                            self.infer_type_args(&info.type_params, &info.params, &arg_types);
                        self.substitute_type(&info.return_type, &subs)
                    } else {
                        self.resolve_type(&info.return_type)
                    };
                    Ok(TypedExpression {
                        kind: TypedExprKind::FunctionCall {
                            function: method.clone(),
                            args: typed_args,
                        },
                        ty: ret_type,
                        span: *span,
                    })
                } else {
                    // Unknown method on type
                    if !type_name.is_empty() {
                        self.diagnostics.push(Diagnostic::warning(
                            "W3043",
                            format!("type `{}` has no method `{}`", type_name, method),
                            *span,
                        ));
                    }
                    Ok(TypedExpression {
                        kind: TypedExprKind::FunctionCall {
                            function: format!("{}_{}", type_name, method),
                            args: typed_args,
                        },
                        ty: Type::Unknown,
                        span: *span,
                    })
                }
            }

            Expression::MemberAccess {
                object,
                field,
                span,
            } => {
                let typed_obj = self.check_expr(object)?;
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
                    // ptr.value → dereference: inner type of Ptr/MutPtr/RawPtr
                    "value" => {
                        let inner_ty = match &typed_obj.ty {
                            Type::Ptr(inner) | Type::MutPtr(inner) | Type::RawPtr(inner) => {
                                *inner.clone()
                            }
                            _ => {
                                self.diagnostics.push(Diagnostic::error(
                                    "E3050",
                                    format!(
                                        "cannot dereference non-pointer type `{}`",
                                        typed_obj.ty.display_name()
                                    ),
                                    *span,
                                ));
                                Type::Unknown
                            }
                        };
                        Ok(TypedExpression {
                            kind: TypedExprKind::Deref(Box::new(typed_obj)),
                            ty: inner_ty,
                            span: *span,
                        })
                    }
                    _ => {
                        let field_type = self.lookup_field_type(&typed_obj.ty, field);
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
                name, fields, span, ..
            } => {
                let mut typed_fields = Vec::new();
                for (field_name, field_expr) in fields {
                    let typed = self.check_expr(field_expr)?;
                    typed_fields.push((field_name.clone(), typed));
                }

                let ty = self.resolve_type(&AstType::Named(name.clone()));
                Ok(TypedExpression {
                    kind: TypedExprKind::StructLiteral {
                        type_name: name.clone(),
                        fields: typed_fields,
                    },
                    ty,
                    span: *span,
                })
            }

            Expression::EnumVariant {
                enum_name,
                variant,
                payload,
                span,
            } => {
                let typed_payload = match payload {
                    Some(p) => Some(Box::new(self.check_expr(p)?)),
                    None => None,
                };
                let ty = Type::Named(enum_name.clone());
                Ok(TypedExpression {
                    kind: TypedExprKind::EnumVariant {
                        type_name: enum_name.clone(),
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

                for arm in arms {
                    self.push_scope();
                    self.bind_pattern(&arm.pattern, &typed_scrutinee.ty);
                    let typed_body = self.check_expr(&arm.body)?;
                    if result_type == Type::Void && typed_body.ty != Type::Void {
                        result_type = typed_body.ty.clone();
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

                // Determine match kind
                let kind = self.determine_match_kind(&typed_scrutinee.ty, arms);

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
                    .map(|(k, v)| (k.clone(), v.clone()))
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
}
