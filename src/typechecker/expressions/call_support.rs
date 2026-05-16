use super::*;

impl TypeChecker {
    pub(super) fn check_call_signature(
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

    pub(super) fn check_call_signature_with_substitutions(
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

    pub(super) fn report_inference_conflicts(
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

    pub(super) fn explicit_type_arg_substitutions(
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

    pub(super) fn check_generic_bounds_valid(
        &mut self,
        bounds: &std::collections::HashMap<String, BehaviorBound>,
        substitutions: &std::collections::HashMap<String, Type>,
        span: Span,
    ) -> bool {
        let diagnostic_count = self.diagnostics.len();
        self.check_generic_bounds(bounds, substitutions, span);
        self.diagnostics.len() == diagnostic_count
    }

    pub(super) fn explicit_type_args_valid(type_args: &[AstType], type_params: &[String]) -> bool {
        type_args.is_empty() || type_args.len() == type_params.len()
    }

    pub(super) fn generic_type_annotation_arities_valid(&self, ast_type: &AstType) -> bool {
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

    pub(super) fn field_access_type_name(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Struct { name, .. } => Some(name.clone()),
            Type::Named(name) if self.structs.contains_key(name) => Some(name.clone()),
            Type::Ptr(inner) | Type::MutPtr(inner) => self.field_access_type_name(inner),
            _ => None,
        }
    }

    pub(super) fn generic_base_type_name(&self, concrete_name: &str) -> Option<String> {
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

    pub(super) fn unknown_method_expr(
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

    pub(super) fn is_root_std_runtime_call(&self, module: &str, function: &str) -> bool {
        self.is_root_std_import(module)
            && matches!((module, function), ("io", "print") | ("io", "println"))
    }

    pub(super) fn block_satisfies_return(&self, block: &TypedBlock, ret_type: &Type) -> bool {
        if block.ty != Type::Void && self.types_compatible(ret_type, &block.ty) {
            return true;
        }

        self.block_definitely_returns(block)
    }

    pub(super) fn block_definitely_returns(&self, block: &TypedBlock) -> bool {
        block
            .expr
            .as_ref()
            .is_some_and(|expr| self.expr_definitely_returns(expr))
            || block.statements.iter().any(|stmt| match &stmt.kind {
                TypedStatementKind::Expression(expr) => self.expr_definitely_returns(expr),
                TypedStatementKind::VarDecl { .. } => false,
            })
    }

    pub(super) fn expr_definitely_returns(&self, expr: &TypedExpression) -> bool {
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
    pub(super) fn check_function_call_expr(
        &mut self,
        name: &str,
        module: &Option<String>,
        type_args: &[AstType],
        args: &[Expression],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let mut typed_args = Vec::new();
        for arg in args {
            typed_args.push(self.check_expr(arg)?);
        }

        // Look up the function
        let full_name = if let Some(m) = module {
            format!("{}.{}", m, name)
        } else {
            name.to_string()
        };

        let (resolved_name, ret_type) = if let Some(info) = self.functions.get(&full_name).cloned()
        {
            if !info.type_params.is_empty() {
                let (subs, explicit_type_args_valid) = if type_args.is_empty() {
                    let arg_types: Vec<Type> = typed_args.iter().map(|a| a.ty.clone()).collect();
                    let (subs, conflicts) = self.infer_type_args_with_conflicts(
                        &info.type_params,
                        &info.params,
                        &arg_types,
                    );
                    self.report_inference_conflicts("function", &full_name, conflicts, span);
                    (subs, true)
                } else {
                    self.explicit_type_arg_substitutions(
                        "function",
                        &full_name,
                        &info.type_params,
                        type_args,
                        span,
                    )
                };
                let (ret, mangled) = if explicit_type_args_valid {
                    self.check_call_signature_with_substitutions(
                        "function",
                        &full_name,
                        &info.params,
                        &typed_args,
                        &subs,
                        &span,
                    );
                    if self.check_generic_bounds_valid(&info.type_param_bounds, &subs, span) {
                        let ret = self.substitute_type(&info.return_type, &subs);
                        let mangled = self
                            .specialize_generic_function(&full_name, &subs, span)
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
                    }
                } else {
                    (
                        Type::Unknown,
                        self.generic_function_mangled_name(&full_name, &info.type_params, &subs),
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
                        span,
                    ));
                }
                self.check_call_signature("function", &full_name, &info.params, &typed_args, &span);
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
                name.to_string()
            };
            if let Some(info) = self.methods.get(&full_name).cloned() {
                self.check_call_signature("method", &full_name, &info.params, &typed_args, &span);
                (full_name.clone(), self.resolve_type(&info.return_type))
            } else if let Some(info) = self.functions.get(&mangled).cloned() {
                self.check_call_signature("function", &mangled, &info.params, &typed_args, &span);
                (full_name.clone(), self.resolve_type(&info.return_type))
            } else {
                let m = module.as_deref().unwrap_or("");
                self.diagnostics.push(Diagnostic::warning(
                    "W3041",
                    format!("unknown function `{}.{}`, assuming void return", m, name),
                    span,
                ));
                (full_name.clone(), Type::Void)
            }
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E3020",
                format!("undefined function `{}`", name),
                span,
            ));
            (full_name.clone(), Type::Unknown)
        };

        Ok(TypedExpression {
            kind: TypedExprKind::FunctionCall {
                function: resolved_name,
                args: typed_args,
            },
            ty: ret_type,
            span,
        })
    }

    pub(super) fn check_method_call_expr(
        &mut self,
        receiver: &Expression,
        method: &str,
        type_args: &[AstType],
        args: &[Expression],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        // Check if receiver is an imported module name (like `io`)
        // In that case, this is a module-qualified call: io.println(args)
        if let Expression::Identifier {
            name: ref recv_name,
            ..
        } = receiver
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
                        &span,
                    );
                    self.resolve_type(&info.return_type)
                } else {
                    let method_key = Self::method_key(recv_name, method);
                    if let Some(info) = self.methods.get(&method_key).cloned() {
                        self.check_call_signature(
                            "method",
                            &method_key,
                            &info.params,
                            &typed_args,
                            &span,
                        );
                        self.resolve_type(&info.return_type)
                    } else if self.is_root_std_runtime_call(recv_name, method) {
                        Type::Void
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            "E3023",
                            format!("undefined module function `{}.{}`", recv_name, method),
                            span,
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
                    span,
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
            Type::Named(n) | Type::Struct { name: n, .. } | Type::Enum { name: n, .. } => n.clone(),
            _ => String::new(),
        };
        let method_key = Self::method_key(&type_name, method);

        if let Some(info) = self.methods.get(&method_key).cloned() {
            // Found as a type method — handle generics
            let (resolved_method, ret_type) = if !info.type_params.is_empty() {
                let (subs, explicit_type_args_valid) = if type_args.is_empty() {
                    let arg_types: Vec<Type> = typed_args.iter().map(|a| a.ty.clone()).collect();
                    let (subs, conflicts) = self.infer_method_type_args(
                        &method_key,
                        &info.type_params,
                        &info.params,
                        &arg_types,
                    );
                    self.report_inference_conflicts("method", &method_key, conflicts, span);
                    (subs, true)
                } else {
                    self.explicit_type_arg_substitutions(
                        "method",
                        &method_key,
                        &info.type_params,
                        type_args,
                        span,
                    )
                };
                let (ret, mangled) = if explicit_type_args_valid {
                    let saved_self_type = self.current_self_type.clone();
                    self.current_self_type = self.generic_method_self_type(&method_key, &subs);
                    self.check_call_signature_with_substitutions(
                        "method",
                        &method_key,
                        &info.params,
                        &typed_args,
                        &subs,
                        &span,
                    );
                    let (ret, mangled) =
                        if self.check_generic_bounds_valid(&info.type_param_bounds, &subs, span) {
                            let ret = self.substitute_type(&info.return_type, &subs);
                            let mangled = self
                                .specialize_generic_method(&method_key, &subs, span)
                                .unwrap_or_else(|| {
                                    self.generic_function_mangled_name(
                                        &method_key,
                                        &info.type_params,
                                        &subs,
                                    )
                                });
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
                    self.current_self_type = saved_self_type;
                    (ret, mangled)
                } else {
                    (
                        Type::Unknown,
                        self.generic_function_mangled_name(&method_key, &info.type_params, &subs),
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
                        span,
                    ));
                }
                self.check_call_signature("method", &method_key, &info.params, &typed_args, &span);
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
                span,
            })
        } else if let Some(generic_base) = self.generic_base_type_name(&type_name) {
            let generic_method_key = Self::method_key(&generic_base, method);
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
                            span,
                        );
                        (subs, true)
                    } else {
                        self.explicit_type_arg_substitutions(
                            "method",
                            &generic_method_key,
                            &info.type_params,
                            type_args,
                            span,
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
                            &span,
                        );
                        let (ret_type, mangled) = if self.check_generic_bounds_valid(
                            &info.type_param_bounds,
                            &subs,
                            span,
                        ) {
                            let ret_type = self.substitute_type(&info.return_type, &subs);
                            let mangled = self
                                .specialize_generic_method(&generic_method_key, &subs, span)
                                .unwrap_or_else(|| {
                                    self.generic_function_mangled_name(
                                        &generic_method_key,
                                        &info.type_params,
                                        &subs,
                                    )
                                });
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
                        span,
                    })
                } else {
                    if !type_args.is_empty() {
                        self.diagnostics.push(Diagnostic::error(
                            "E5001",
                            format!(
                                "non-generic method `{}` does not accept type arguments",
                                generic_method_key
                            ),
                            span,
                        ));
                    }
                    self.check_call_signature(
                        "method",
                        &generic_method_key,
                        &info.params,
                        &typed_args,
                        &span,
                    );
                    Ok(TypedExpression {
                        kind: TypedExprKind::FunctionCall {
                            function: format!("{}_{}", generic_base, method),
                            args: typed_args,
                        },
                        ty: self.resolve_type(&info.return_type),
                        span,
                    })
                }
            } else {
                self.unknown_method_expr(&type_name, method, typed_args, span)
            }
        } else if let Some(info) = self.functions.get(method).cloned() {
            // UFC: x.f(args) -> f(x, args)
            let (resolved_function, ret_type) = if !info.type_params.is_empty() {
                let (subs, explicit_type_args_valid) = if type_args.is_empty() {
                    let arg_types: Vec<Type> = typed_args.iter().map(|a| a.ty.clone()).collect();
                    let (subs, conflicts) = self.infer_type_args_with_conflicts(
                        &info.type_params,
                        &info.params,
                        &arg_types,
                    );
                    self.report_inference_conflicts("function", method, conflicts, span);
                    (subs, true)
                } else {
                    self.explicit_type_arg_substitutions(
                        "function",
                        method,
                        &info.type_params,
                        type_args,
                        span,
                    )
                };
                let (ret, mangled) = if explicit_type_args_valid {
                    self.check_call_signature_with_substitutions(
                        "function",
                        method,
                        &info.params,
                        &typed_args,
                        &subs,
                        &span,
                    );
                    if self.check_generic_bounds_valid(&info.type_param_bounds, &subs, span) {
                        let ret = self.substitute_type(&info.return_type, &subs);
                        let mangled = self
                            .specialize_generic_function(method, &subs, span)
                            .unwrap_or_else(|| {
                                self.generic_function_mangled_name(method, &info.type_params, &subs)
                            });
                        (ret, mangled)
                    } else {
                        (
                            Type::Unknown,
                            self.generic_function_mangled_name(method, &info.type_params, &subs),
                        )
                    }
                } else {
                    (
                        Type::Unknown,
                        self.generic_function_mangled_name(method, &info.type_params, &subs),
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
                        span,
                    ));
                }
                self.check_call_signature("function", method, &info.params, &typed_args, &span);
                (method.to_string(), self.resolve_type(&info.return_type))
            };
            Ok(TypedExpression {
                kind: TypedExprKind::FunctionCall {
                    function: resolved_function,
                    args: typed_args,
                },
                ty: ret_type,
                span,
            })
        } else {
            self.unknown_method_expr(&type_name, method, typed_args, span)
        }
    }
}
