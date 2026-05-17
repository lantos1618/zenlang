use super::*;

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
        if !self.is_import(recv_name) {
            return None;
        }

        Some(self.check_module_qualified_method_call(recv_name, method, type_args, args, span))
    }

    fn check_module_qualified_method_call(
        &mut self,
        recv_name: &str,
        method: &str,
        type_args: &[AstType],
        args: &[Expression],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let mut typed_args = Vec::new();
        for arg in args {
            typed_args.push(self.check_expr(arg)?);
        }

        let mangled = format!("{}_{}", recv_name, method);
        let ret_type = if let Some(info) = self.functions.get(&mangled).cloned() {
            self.reject_module_call_type_args(
                "function",
                &format!("{}.{}", recv_name, method),
                type_args,
                span,
            );
            self.check_call_signature("function", &mangled, &info.params, &typed_args, &span);
            self.resolve_type(&info.return_type)
        } else {
            self.check_module_method_fallback(recv_name, method, type_args, &typed_args, span)
        };

        Ok(TypedExpression {
            kind: TypedExprKind::FunctionCall {
                function: mangled,
                args: typed_args,
            },
            ty: ret_type,
            span,
        })
    }

    fn check_module_method_fallback(
        &mut self,
        recv_name: &str,
        method: &str,
        type_args: &[AstType],
        typed_args: &[TypedExpression],
        span: Span,
    ) -> Type {
        let method_key = Self::method_key(recv_name, method);
        if let Some(info) = self.methods.get(&method_key).cloned() {
            self.reject_module_call_type_args("method", &method_key, type_args, span);
            self.check_call_signature("method", &method_key, &info.params, typed_args, &span);
            return self.resolve_type(&info.return_type);
        }

        if self.is_root_std_runtime_call(recv_name, method) {
            self.reject_module_call_type_args(
                "function",
                &format!("{}.{}", recv_name, method),
                type_args,
                span,
            );
            return Type::Void;
        }

        self.diagnostics.push(Diagnostic::error(
            "E3023",
            format!("undefined module function `{}.{}`", recv_name, method),
            span,
        ));
        Type::Unknown
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
