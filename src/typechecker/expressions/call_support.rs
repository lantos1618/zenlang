use super::*;
use crate::typechecker::gated_intrinsics::GatedIntrinsic;

impl TypeChecker {
    pub(super) fn check_function_call_expr(
        &mut self,
        name: &str,
        module: &Option<String>,
        type_args: &[AstType],
        args: &[Expression],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let full_name = if let Some(m) = module {
            format!("{}.{}", m, name)
        } else {
            name.to_string()
        };

        if module.as_deref() == Some(GatedIntrinsic::INTRINSIC_MODULE) {
            let typed_args = self.check_call_args(args)?;
            if let Ok(gated) = name.parse::<GatedIntrinsic>() {
                self.diagnostics
                    .push(Diagnostic::error("E0203", gated.gate_message(), span));
                return Ok(TypedExpression {
                    kind: TypedExprKind::FunctionCall {
                        function: full_name,
                        args: typed_args,
                    },
                    ty: Type::Unknown,
                    span,
                });
            }
        }

        let (resolved_name, ret_type, typed_args) = if let Some(info) =
            self.functions.get(&full_name).cloned()
        {
            if !info.type_params.is_empty() {
                let typed_args = self.check_call_args(args)?;
                let (resolved_name, ret_type) = self.resolve_generic_function_call(
                    &full_name,
                    &info,
                    type_args,
                    &typed_args,
                    span,
                );
                (resolved_name, ret_type, typed_args)
            } else {
                let typed_args = self.check_call_args_with_param_context(args, &info.params)?;
                if !type_args.is_empty() {
                    self.diagnostics.push(Diagnostic::error(
                        "E5002",
                        format!(
                            "non-generic function `{}` does not accept type arguments",
                            full_name
                        ),
                        span,
                    ));
                }
                self.check_call_signature("function", &full_name, &info.params, &typed_args, &span);
                (
                    full_name.clone(),
                    self.resolve_type(&info.return_type),
                    typed_args,
                )
            }
        } else if name == "cast" && args.len() == 1 && !type_args.is_empty() {
            let typed_args = self.check_call_args(args)?;
            // cast(expr, Type) — handled specially
            (
                full_name.clone(),
                self.resolve_type(&type_args[0]),
                typed_args,
            )
        } else if module.is_some() {
            let typed_args = self.check_call_args(args)?;
            // Try looking up module-qualified names in methods/functions maps
            let mangled = if let Some(m) = module {
                format!("{}_{}", m, name)
            } else {
                name.to_string()
            };
            if let Some(info) = self.methods.get(&full_name).cloned() {
                self.reject_direct_module_call_type_args("method", &full_name, type_args, span);
                self.check_call_signature("method", &full_name, &info.params, &typed_args, &span);
                (
                    full_name.clone(),
                    self.resolve_type(&info.return_type),
                    typed_args,
                )
            } else if let Some(info) = self.functions.get(&mangled).cloned() {
                self.reject_direct_module_call_type_args("function", &full_name, type_args, span);
                self.check_call_signature("function", &mangled, &info.params, &typed_args, &span);
                (
                    full_name.clone(),
                    self.resolve_type(&info.return_type),
                    typed_args,
                )
            } else {
                self.reject_direct_module_call_type_args("function", &full_name, type_args, span);
                let m = module.as_deref().unwrap_or("");
                self.diagnostics.push(Diagnostic::warning(
                    "W3041",
                    format!("unknown function `{}.{}`, assuming void return", m, name),
                    span,
                ));
                (full_name.clone(), Type::Void, typed_args)
            }
        } else {
            let typed_args = self.check_call_args(args)?;
            self.diagnostics.push(Diagnostic::error(
                "E3020",
                format!("undefined function `{}`", name),
                span,
            ));
            (full_name.clone(), Type::Unknown, typed_args)
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

    fn check_call_args(&mut self, args: &[Expression]) -> Result<Vec<TypedExpression>, Diagnostic> {
        args.iter().map(|arg| self.check_expr(arg)).collect()
    }

    fn check_call_args_with_param_context(
        &mut self,
        args: &[Expression],
        params: &[(String, AstType)],
    ) -> Result<Vec<TypedExpression>, Diagnostic> {
        if args.len() != params.len() {
            return self.check_call_args(args);
        }

        args.iter()
            .zip(params)
            .map(|(arg, (_, param_ty))| {
                let expected = self.resolve_type(param_ty);
                self.check_expr_with_expected_type(
                    arg,
                    (expected != Type::Unknown).then_some(expected),
                )
            })
            .collect()
    }

    fn check_expr_with_expected_type(
        &mut self,
        expr: &Expression,
        expected: Option<Type>,
    ) -> Result<TypedExpression, Diagnostic> {
        let Some(expected) = expected else {
            return self.check_expr(expr);
        };

        let saved_return_type = self.current_return_type.take();
        self.current_return_type = Some(expected);
        let checked = self.check_expr(expr);
        self.current_return_type = saved_return_type;
        checked
    }

    fn reject_direct_module_call_type_args(
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
            "E5002",
            format!(
                "non-generic {} `{}` does not accept type arguments",
                kind, name
            ),
            span,
        ));
    }
}
