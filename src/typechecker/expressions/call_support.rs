use super::*;
use crate::root_spelling::AT_BUILTIN_ROOT;

impl TypeChecker {
    pub(super) fn check_function_call_expr(
        &mut self,
        name: &str,
        module: &Option<String>,
        type_args: &[AstType],
        args: &[Expression],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_args = self.check_exprs(args)?;

        let full_name = module
            .as_ref()
            .map_or_else(|| name.to_string(), |module| format!("{module}.{name}"));

        if module.as_deref() == Some(AT_BUILTIN_ROOT) {
            self.push_error(
                E0203,
                format!(
                    "`@builtin.{name}` is gated until the Zen stdlib compiler facade defines it"
                ),
                span,
            );
            return Ok(typed_call_expr(full_name, typed_args, Type::Unknown, span));
        }

        let (resolved_name, ret_type) = if let Some(info) = self.functions.get(&full_name).cloned()
        {
            self.resolve_callable_call("function", &full_name, &info, type_args, &typed_args, span)
        } else if name == "cast" && typed_args.len() == 1 && !type_args.is_empty() {
            (full_name.clone(), self.resolve_type(&type_args[0]))
        } else if let Some(module) = module {
            let mangled = format!("{module}_{name}");
            if let Some(info) = self.methods.get(&full_name).cloned() {
                self.reject_nongeneric_type_args("method", &full_name, type_args, span);
                self.check_call_signature("method", &full_name, &info.params, &typed_args, &span);
                (full_name.clone(), self.resolve_type(&info.return_type))
            } else if let Some(info) = self.functions.get(&mangled).cloned() {
                self.reject_nongeneric_type_args("function", &full_name, type_args, span);
                self.check_call_signature("function", &mangled, &info.params, &typed_args, &span);
                (full_name.clone(), self.resolve_type(&info.return_type))
            } else {
                self.reject_nongeneric_type_args("function", &full_name, type_args, span);
                self.push_error(
                    E3023,
                    format!("undefined module function `{full_name}`"),
                    span,
                );
                (full_name.clone(), Type::Unknown)
            }
        } else {
            self.push_error(E3020, format!("undefined function `{}`", name), span);
            (full_name.clone(), Type::Unknown)
        };

        Ok(typed_call_expr(resolved_name, typed_args, ret_type, span))
    }
}
