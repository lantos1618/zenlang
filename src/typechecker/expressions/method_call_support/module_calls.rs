use super::*;
use crate::typechecker::method_signature_key;

impl TypeChecker {
    pub(super) fn try_module_qualified_method_call(
        &mut self,
        receiver: &Expression,
        method: &str,
        type_args: &[AstType],
        args: &[Expression],
        span: Span,
    ) -> Option<Result<TypedExpression, Diagnostic>> {
        let Expression::Identifier {
            name: recv_name, ..
        } = receiver
        else {
            return None;
        };
        if !self.imports.contains(recv_name) {
            return None;
        }

        let mut typed_args = Vec::new();
        for arg in args {
            match self.check_expr(arg) {
                Ok(arg) => typed_args.push(arg),
                Err(diagnostic) => return Some(Err(diagnostic)),
            }
        }

        let mangled = format!("{}_{}", recv_name, method);
        let method_key = method_signature_key(recv_name, method);
        let (resolved_name, ret_type) = if let Some(info) = self.functions.get(&mangled).cloned() {
            if info.type_params.is_empty() {
                self.reject_nongeneric_type_args(
                    "function",
                    &format!("{}.{}", recv_name, method),
                    type_args,
                    span,
                );
                self.check_call_signature("function", &mangled, &info.params, &typed_args, &span);
                (mangled.clone(), self.resolve_type(&info.return_type))
            } else {
                // Generic stdlib functions (e.g. `math.max<T>`) specialize via the
                // same path as ordinary generic calls.
                self.resolve_callable_call("function", &mangled, &info, type_args, &typed_args, span)
            }
        } else if let Some(info) = self.methods.get(&method_key).cloned() {
            self.reject_nongeneric_type_args("method", &method_key, type_args, span);
            self.check_call_signature("method", &method_key, &info.params, &typed_args, &span);
            (mangled.clone(), self.resolve_type(&info.return_type))
        } else {
            self.push_error(
                E3023,
                format!("undefined module function `{}.{}`", recv_name, method),
                span,
            );
            (mangled.clone(), Type::Unknown)
        };

        Some(Ok(typed_call_expr(
            resolved_name,
            typed_args,
            ret_type,
            span,
        )))
    }
}
