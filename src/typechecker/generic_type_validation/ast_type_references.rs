use super::*;

impl TypeChecker {
    #[cfg(test)]
    pub(in crate::typechecker) fn collect_ast_type_reference_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<AstTypeReferenceValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_type_reference_validation_task(decl, &mut tasks);
        }
        tasks
    }

    pub(in crate::typechecker) fn push_ast_type_reference_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstTypeReferenceValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Struct {
                type_params,
                fields,
            }),
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Enum {
                type_params,
                variants,
            }),
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Function {
                type_params,
                params,
                return_type,
                body,
            }),
            Declaration::Method {
                type_params,
                params,
                return_type,
                body,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Method {
                type_params,
                params,
                return_type,
                body,
            }),
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Behavior {
                type_params,
                methods,
            }),
            Declaration::ImplBlock { methods, .. } => {
                tasks.push(AstTypeReferenceValidationTask::ImplBlock { methods });
            }
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(AstTypeReferenceValidationTask::TopLevelExpr { expr });
            }
            _ => {}
        }
    }

    pub(in crate::typechecker) fn validate_ast_type_reference_tasks(
        &mut self,
        tasks: &[AstTypeReferenceValidationTask<'_>],
    ) {
        for task in tasks {
            match task {
                AstTypeReferenceValidationTask::Struct {
                    type_params,
                    fields,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for field in *fields {
                        self.validate_generic_type_ref_bounds(&field.ty, &scoped, field.span);
                        if let Some(default) = &field.default {
                            self.validate_generic_expr_type_references(default, &scoped);
                        }
                    }
                }
                AstTypeReferenceValidationTask::Enum {
                    type_params,
                    variants,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for variant in *variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_generic_type_ref_bounds(payload, &scoped, variant.span);
                        }
                    }
                }
                AstTypeReferenceValidationTask::Function {
                    type_params,
                    params,
                    return_type,
                    body,
                } => {
                    self.validate_ast_callable_type_references(
                        type_params,
                        params,
                        return_type,
                        body,
                        Span::dummy(),
                    );
                }
                AstTypeReferenceValidationTask::Method {
                    type_params,
                    params,
                    return_type,
                    body,
                } => {
                    self.validate_ast_callable_type_references(
                        type_params,
                        params,
                        return_type,
                        body,
                        Span::dummy(),
                    );
                }
                AstTypeReferenceValidationTask::Behavior {
                    type_params,
                    methods,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for method in *methods {
                        for param in &method.params {
                            self.validate_generic_type_ref_bounds(&param.ty, &scoped, param.span);
                        }
                        if let Some(return_type) = &method.return_type {
                            self.validate_generic_type_ref_bounds(
                                return_type,
                                &scoped,
                                method.span,
                            );
                        }
                        if let Some(default_body) = &method.default_body {
                            self.validate_generic_expr_type_references(default_body, &scoped);
                        }
                    }
                }
                AstTypeReferenceValidationTask::ImplBlock { methods } => {
                    for method in *methods {
                        if let Declaration::Function {
                            type_params,
                            params,
                            return_type,
                            body,
                            ..
                        } = method
                        {
                            self.validate_ast_callable_type_references(
                                type_params,
                                params,
                                return_type,
                                body,
                                method.span(),
                            );
                        }
                    }
                }
                AstTypeReferenceValidationTask::TopLevelExpr { expr } => {
                    self.validate_generic_expr_type_references(expr, &HashSet::new());
                }
            }
        }
    }

    fn validate_ast_callable_type_references(
        &mut self,
        type_params: &[ast::TypeParam],
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        return_span: Span,
    ) {
        let scoped = type_param_name_set(type_params);
        for param in params {
            self.validate_generic_type_ref_bounds(&param.ty, &scoped, param.span);
        }
        if let Some(return_type) = return_type {
            self.validate_generic_type_ref_bounds(return_type, &scoped, return_span);
        }
        self.validate_generic_expr_type_references(body, &scoped);
    }
}
