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
            return Ok(
                self.check_builtin_intrinsic_call(name, &full_name, type_args, typed_args, span)
            );
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

    /// Resolve `@builtin.<name>(...)`. The compiler owns the primitives (the
    /// registry in `crate::intrinsics`); the usable subset lowers to a
    /// `TypedExprKind::Intrinsic` node the C backend emits directly. Primitives
    /// whose semantics aren't settled yet (syscalls, atomics, async, comptime
    /// type-match) stay gated.
    fn check_builtin_intrinsic_call(
        &mut self,
        name: &str,
        full_name: &str,
        type_args: &[AstType],
        typed_args: Vec<TypedExpression>,
        span: Span,
    ) -> TypedExpression {
        let gated = name.starts_with("syscall")
            || name.starts_with("atomic_")
            || matches!(
                name,
                "fence" | "async_enqueue" | "async_yield" | "type_match"
            );
        if gated {
            self.push_error(
                E0203,
                format!(
                    "`@builtin.{name}` is gated until the Zen stdlib compiler facade defines it"
                ),
                span,
            );
            return typed_call_expr(full_name.to_string(), typed_args, Type::Unknown, span);
        }

        // Typed memory/type intrinsics: `load<T>(ptr) -> T`, `sizeof<T>() -> usize`.
        if name == "load" && type_args.len() == 1 && typed_args.len() == 1 {
            let ty = self.resolve_type(&type_args[0]);
            return typed_expr(
                TypedExprKind::Intrinsic {
                    name: name.to_string(),
                    args: typed_args,
                },
                ty,
                span,
            );
        }
        if matches!(name, "sizeof" | "alignof") && type_args.len() == 1 {
            let marker = typed_expr(
                TypedExprKind::IntLiteral(0),
                self.resolve_type(&type_args[0]),
                span,
            );
            return typed_expr(
                TypedExprKind::Intrinsic {
                    name: name.to_string(),
                    args: vec![marker],
                },
                Type::Usize,
                span,
            );
        }

        match crate::intrinsics::check_intrinsic_call(name, typed_args.len()) {
            Some(Ok(ret)) => {
                let ty = self.resolve_type(&ret);
                typed_expr(
                    TypedExprKind::Intrinsic {
                        name: name.to_string(),
                        args: typed_args,
                    },
                    ty,
                    span,
                )
            }
            Some(Err(_)) => {
                let expected = crate::intrinsics::get_intrinsic(name).map_or(0, |i| i.params.len());
                self.push_error(
                    E3021,
                    format!(
                        "intrinsic `{full_name}` expects {expected} arguments, found {}",
                        typed_args.len()
                    ),
                    span,
                );
                typed_expr(
                    TypedExprKind::Intrinsic {
                        name: name.to_string(),
                        args: typed_args,
                    },
                    Type::Unknown,
                    span,
                )
            }
            None => {
                self.push_error(
                    E3023,
                    format!("unknown compiler intrinsic `{full_name}`"),
                    span,
                );
                typed_expr(
                    TypedExprKind::Intrinsic {
                        name: name.to_string(),
                        args: typed_args,
                    },
                    Type::Unknown,
                    span,
                )
            }
        }
    }
}
