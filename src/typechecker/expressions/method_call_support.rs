use super::gated_methods::GatedMethod;
use super::*;

mod generic_methods;
mod module_calls;

impl TypeChecker {
    pub(super) fn check_method_call_expr(
        &mut self,
        receiver: &Expression,
        method: &str,
        type_args: &[AstType],
        args: &[Expression],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        if let Some(module_call) =
            self.try_module_qualified_method_call(receiver, method, type_args, args, span)
        {
            return module_call;
        }

        let typed_receiver = self.check_expr(receiver)?;

        if let Ok(gated_method) = method.parse::<GatedMethod>() {
            return Err(gated_method.diagnostic(span));
        }

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
                self.resolve_generic_method_call(&method_key, &info, type_args, &typed_args, span)
            } else {
                if !type_args.is_empty() {
                    self.diagnostics.push(Diagnostic::error(
                        "E5002",
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
                    let (mangled, ret_type) = self.resolve_generic_method_call(
                        &generic_method_key,
                        &info,
                        type_args,
                        &typed_args,
                        span,
                    );
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
                            "E5002",
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
                self.resolve_generic_function_call(method, &info, type_args, &typed_args, span)
            } else {
                if !type_args.is_empty() {
                    self.diagnostics.push(Diagnostic::error(
                        "E5002",
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
