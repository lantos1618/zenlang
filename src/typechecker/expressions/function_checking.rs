use super::*;

impl TypeChecker {
    pub(crate) fn check_function(
        &mut self,
        name: &str,
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: &Span,
    ) -> Result<TypedFunction, Diagnostic> {
        let return_annotation_valid = return_type
            .as_ref()
            .is_none_or(|ty| self.generic_type_annotation_arities_valid(ty));
        let ret_type = return_type
            .as_ref()
            .map_or(Type::Void, |ty| self.resolve_type(ty));

        let saved_defers = std::mem::take(&mut self.pending_defers);
        let saved_return_type = self.current_return_type.replace(ret_type.clone());

        let checked = self.with_scope(|checker| {
            let mut typed_params = Vec::new();
            for param in params {
                let ty = checker.resolve_type(&param.ty);
                checker.define_var(&param.name, ty.clone());
                typed_params.push(TypedParam {
                    name: param.name.clone(),
                    ty,
                    span: param.span,
                });
            }

            Ok((typed_params, checker.check_expr(body)?))
        });
        self.current_return_type = saved_return_type;

        let (typed_params, typed_body) = match checked {
            Ok(checked) => checked,
            Err(diagnostic) => {
                self.pending_defers = saved_defers;
                return Err(diagnostic);
            }
        };
        let mut body_block = match typed_body.kind {
            TypedExprKind::Block(block) => block,
            _ => typed_block_from_expr(typed_body),
        };

        // Untyped numeric-literal results adopt the declared numeric return type
        // (a `u64` function whose branch is the literal `1`), recursing through
        // match arms / if branches / block tails.
        if ret_type.is_integer() || ret_type.is_float() {
            if let Some(expr) = &mut body_block.expr {
                coerce_result_to_numeric_type(expr, &ret_type);
                body_block.ty = expr.ty.clone();
            }
        }

        let result = (|| -> Result<TypedFunction, Diagnostic> {
            if return_annotation_valid && ret_type != Type::Void && ret_type != Type::Never {
                if let Some(expr) = &body_block.expr {
                    if expr.ty != Type::Never && !self.types_compatible(&ret_type, &expr.ty) {
                        let (expected, actual) = type_display_pair(&ret_type, &expr.ty);
                        return Err(Diagnostic::error_code(
                            E3030,
                            format!(
                                "return type mismatch: expected `{expected}`, found `{actual}`"
                            ),
                            expr.span,
                        ));
                    }
                }

                let body_satisfies_return = body_block.ty != Type::Void
                    && self.types_compatible(&ret_type, &body_block.ty)
                    || self.block_definitely_returns(&body_block);
                if !body_satisfies_return {
                    return Err(Diagnostic::error_code(
                        E3031,
                        format!(
                            "function `{name}` must return `{}` on all non-error paths",
                            ret_type.display_name()
                        ),
                        *span,
                    ));
                }
            }

            let mut defers: Vec<TypedExpression> = self.pending_defers.drain(..).collect();
            defers.reverse();

            Ok(TypedFunction {
                name: name.to_string(),
                params: typed_params,
                return_type: ret_type,
                body: body_block,
                defers,
                span: *span,
            })
        })();
        self.pending_defers = saved_defers;
        result
    }
}

/// Push an expected numeric return type into untyped numeric-literal results,
/// recursing through `match` arms, `if` branches, and block tails so a function
/// whose branches are bare literals (e.g. `1` in a `u64` function) type-checks
/// without an explicit cast. Only literal leaves are retyped.
fn coerce_result_to_numeric_type(expr: &mut TypedExpression, expected: &Type) {
    if !(expected.is_integer() || expected.is_float()) {
        return;
    }
    match &mut expr.kind {
        TypedExprKind::IntLiteral(_) if expr.ty.is_integer() && expected.is_integer() => {
            expr.ty = expected.clone();
        }
        TypedExprKind::FloatLiteral(_) if expr.ty.is_float() && expected.is_float() => {
            expr.ty = expected.clone();
        }
        TypedExprKind::Match { arms, .. } => {
            for arm in arms.iter_mut() {
                coerce_block_result_to_numeric_type(&mut arm.body, expected);
            }
            if arms.iter().all(|arm| &arm.body.ty == expected) {
                expr.ty = expected.clone();
            }
        }
        TypedExprKind::Block(block) => {
            coerce_block_result_to_numeric_type(block, expected);
            expr.ty = block.ty.clone();
        }
        _ => {}
    }
}

fn coerce_block_result_to_numeric_type(block: &mut TypedBlock, expected: &Type) {
    if let Some(expr) = &mut block.expr {
        coerce_result_to_numeric_type(expr, expected);
        block.ty = expr.ty.clone();
    }
}
