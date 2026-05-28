use super::*;
use crate::typechecker::method_signature_key;

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

        if method == "raise" {
            return Err(Diagnostic::error_code(
                E3054,
                "`.raise()` is gated until Result propagation typing and lowering are implemented",
                span,
            ));
        }

        let mut typed_args = vec![typed_receiver.clone()];
        typed_args.extend(self.check_exprs(args)?);

        let type_name = typed_receiver.ty.nominal_name().unwrap_or("").to_string();
        let generic_base_type =
            self.structs
                .iter()
                .filter_map(|(name, info)| (!info.type_params.is_empty()).then_some(name.as_str()))
                .chain(self.enums.iter().filter_map(|(name, info)| {
                    (!info.type_params.is_empty()).then_some(name.as_str())
                }))
                .find(|name| self.concrete_type_name_matches_generic(&type_name, name))
                .map(str::to_string);

        let method_candidates = std::iter::once((type_name.as_str(), false))
            .chain(generic_base_type.as_deref().map(|name| (name, true)));
        for (candidate_type, generic_base) in method_candidates {
            let method_key = self
                .behavior_specialized_method_key(candidate_type, method)
                .unwrap_or_else(|| method_signature_key(candidate_type, method));
            let Some(info) = self.methods.get(&method_key).cloned() else {
                continue;
            };
            let (resolved_method, ret_type) = self.resolve_callable_call(
                "method",
                &method_key,
                &info,
                type_args,
                &typed_args,
                span,
            );
            let resolved_method = if generic_base && info.type_params.is_empty() {
                format!("{}_{}", candidate_type, method)
            } else {
                resolved_method
            };
            return Ok(typed_call_expr(resolved_method, typed_args, ret_type, span));
        }

        if let Some(info) = self.functions.get(method).cloned() {
            let (resolved_function, ret_type) =
                self.resolve_callable_call("function", method, &info, type_args, &typed_args, span);
            Ok(typed_call_expr(
                resolved_function,
                typed_args,
                ret_type,
                span,
            ))
        } else {
            if !type_name.is_empty() {
                self.push_error(
                    E3043,
                    format!("type `{type_name}` has no method `{method}`"),
                    span,
                );
            }
            Ok(typed_call_expr(
                format!("{}_{}", type_name, method),
                typed_args,
                Type::Unknown,
                span,
            ))
        }
    }
}
