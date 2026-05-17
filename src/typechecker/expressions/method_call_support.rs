use super::*;

impl TypeChecker {
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
                    self.reject_module_call_type_args(
                        "function",
                        &format!("{}.{}", recv_name, method),
                        type_args,
                        span,
                    );
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
                        self.reject_module_call_type_args("method", &method_key, type_args, span);
                        self.check_call_signature(
                            "method",
                            &method_key,
                            &info.params,
                            &typed_args,
                            &span,
                        );
                        self.resolve_type(&info.return_type)
                    } else if self.is_root_std_runtime_call(recv_name, method) {
                        self.reject_module_call_type_args(
                            "function",
                            &format!("{}.{}", recv_name, method),
                            type_args,
                            span,
                        );
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
        let method_key = self
            .behavior_specialized_method_key(&type_name, method)
            .unwrap_or_else(|| Self::method_key(&type_name, method));

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
                    let inferred_type_args_valid =
                        self.report_inference_conflicts("method", &method_key, conflicts, span);
                    (subs, inferred_type_args_valid)
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
                (method_key.clone(), self.resolve_type(&info.return_type))
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
                        let inferred_type_args_valid = self.report_inference_conflicts(
                            "method",
                            &generic_method_key,
                            conflicts,
                            span,
                        );
                        (subs, inferred_type_args_valid)
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
                    let inferred_type_args_valid =
                        self.report_inference_conflicts("function", method, conflicts, span);
                    (subs, inferred_type_args_valid)
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

    fn reject_module_call_type_args(
        &mut self,
        kind: &str,
        name: &str,
        type_args: &[AstType],
        span: Span,
    ) {
        if type_args.is_empty() {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            "E5001",
            format!(
                "non-generic {} `{}` does not accept type arguments",
                kind, name
            ),
            span,
        ));
    }
}
