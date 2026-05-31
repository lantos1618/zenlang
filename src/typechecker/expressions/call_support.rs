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
            let (resolved, ret) = self
                .resolve_callable_call("function", &full_name, &info, type_args, &typed_args, span);
            // A call to an `@async` function yields a `Future<T>`, not `T`; the
            // value is unwrapped to `T` only by `@await`.
            let ret = if info.is_async {
                Type::Future(Box::new(ret))
            } else {
                ret
            };
            (resolved, ret)
        } else if name == "cast" && typed_args.len() == 1 && !type_args.is_empty() {
            (full_name.clone(), self.resolve_type(&type_args[0]))
        } else if name == "pending_then_ready" && module.is_none() && typed_args.len() == 2 {
            // A compiler-provided *test* future (ASYNC_PLAN.md milestone 2): it
            // returns Pending for the first `n` polls, then Ready(`value`). It is
            // the deterministic Pending source used to prove genuine
            // suspend/resume before a real I/O readiness source (milestone 3)
            // exists. `pending_then_ready(n, value) -> Future<i32>`.
            (full_name.clone(), Type::Future(Box::new(Type::I32)))
        } else if name == "block_on" && module.is_none() && typed_args.len() == 1 {
            // `block_on(fut)` drives a future to completion and yields its value.
            // A compiler-provided driver (ASYNC_PLAN.md milestone 1) standing in
            // for the stdlib scheduler; legal from any (sync) context.
            let Type::Future(value_ty) = &typed_args[0].ty else {
                self.push_error(
                    E3081,
                    format!(
                        "`block_on` expects a future, found `{}`",
                        typed_args[0].ty.display_name()
                    ),
                    span,
                );
                return Ok(typed_call_expr(full_name, typed_args, Type::Unknown, span));
            };
            (full_name.clone(), (**value_ty).clone())
        } else if let Some(module) = module {
            let mangled = format!("{module}_{name}");
            if let Some(info) = self.methods.get(&full_name).cloned() {
                self.reject_nongeneric_type_args("method", &full_name, type_args, span);
                self.check_call_signature("method", &full_name, &info.params, &typed_args, &span);
                (full_name.clone(), self.resolve_type(&info.return_type))
            } else if let Some(info) = self.functions.get(&mangled).cloned() {
                // Generic stdlib functions (e.g. `math.max<T>`) specialize via
                // the same path as ordinary generic calls.
                self.resolve_callable_call(
                    "function",
                    &mangled,
                    &info,
                    type_args,
                    &typed_args,
                    span,
                )
            } else {
                self.reject_nongeneric_type_args("function", &full_name, type_args, span);
                self.push_error(
                    E3023,
                    format!("undefined module function `{full_name}`"),
                    span,
                );
                (full_name.clone(), Type::Unknown)
            }
        } else if let Some(Type::Function { params, ret }) =
            self.lookup_var_info(name).map(|info| info.ty.clone())
        {
            // Indirect call through a function-typed local or parameter
            // (a higher-order callback like `vec_map`'s `f`). Emits as a plain
            // `f(args)` call against the function-pointer variable in C.
            self.reject_nongeneric_type_args("function value", name, type_args, span);
            self.check_call_signature_types("function value", name, &params, &typed_args, &span);
            (full_name.clone(), *ret)
        } else {
            self.push_error(E3020, format!("undefined function `{}`", name), span);
            (full_name.clone(), Type::Unknown)
        };

        Ok(typed_call_expr(resolved_name, typed_args, ret_type, span))
    }

    /// Resolve `@builtin.<name>(...)`. The compiler owns the primitives (the
    /// registry in `crate::intrinsics`); the usable subset lowers to a
    /// `TypedExprKind::Intrinsic` node the C backend emits directly. `syscall*`,
    /// `atomic_*`, and `fence` are settled OS/hardware hooks with full C lowering,
    /// exposed so the stdlib can build sys/io/concurrency on them (the recommended
    /// path is the `stdlib/compiler.zen` facade). Still gated: the async runtime
    /// hooks (mid-build, see ASYNC_PLAN.md) and comptime `type_match` (semantics
    /// not settled).
    fn check_builtin_intrinsic_call(
        &mut self,
        name: &str,
        full_name: &str,
        type_args: &[AstType],
        typed_args: Vec<TypedExpression>,
        span: Span,
    ) -> TypedExpression {
        let gated = matches!(name, "async_enqueue" | "async_yield" | "type_match");
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
            self.ensure_specialized_type_refs_for_type(&ty, span);
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
            let ty = self.resolve_type(&type_args[0]);
            self.ensure_specialized_type_refs_for_type(&ty, span);
            let marker = typed_expr(TypedExprKind::IntLiteral(0), ty, span);
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
