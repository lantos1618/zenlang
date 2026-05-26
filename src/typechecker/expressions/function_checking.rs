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
            .map(|ty| self.resolve_type(ty))
            .unwrap_or(Type::Void);

        let saved_return_type = self.current_return_type.clone();
        let saved_defers = std::mem::take(&mut self.pending_defers);
        self.current_return_type = Some(ret_type.clone());

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
        let body_block = match typed_body.kind {
            TypedExprKind::Block(block) => block,
            _ => TypedBlock {
                ty: typed_body.ty.clone(),
                span: typed_body.span,
                statements: Vec::new(),
                expr: Some(Box::new(typed_body)),
            },
        };

        let result = (|| -> Result<TypedFunction, Diagnostic> {
            if return_annotation_valid && ret_type != Type::Void && ret_type != Type::Never {
                if let Some(expr) = &body_block.expr {
                    if expr.ty != Type::Never && !self.types_compatible(&ret_type, &expr.ty) {
                        return Err(Diagnostic::error_code(
                            crate::error::CompilerDiagnosticCode::E3030,
                            format!(
                                "return type mismatch: expected `{}`, found `{}`",
                                ret_type.display_name(),
                                expr.ty.display_name()
                            ),
                            expr.span,
                        ));
                    }
                }

                if !self.block_satisfies_return(&body_block, &ret_type) {
                    return Err(Diagnostic::error_code(
                        crate::error::CompilerDiagnosticCode::E3031,
                        format!(
                            "function `{}` must return `{}` on all non-error paths",
                            name,
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
