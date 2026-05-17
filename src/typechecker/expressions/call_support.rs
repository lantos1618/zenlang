use super::*;

impl TypeChecker {
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
                    let inferred_type_args_valid =
                        self.report_inference_conflicts("function", &full_name, conflicts, span);
                    (subs, inferred_type_args_valid)
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
}
