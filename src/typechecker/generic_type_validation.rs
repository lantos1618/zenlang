use super::*;

mod ast_tasks;
mod resolver_type_references;

impl TypeChecker {
    pub(super) fn validate_ast_type_reference_tasks(
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

    pub(super) fn validation_symbol_name(
        symbols: Option<&SymbolTable>,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) -> String {
        symbols
            .map(|symbols| Self::resolver_symbol_name_for(symbols, namespace, name, span))
            .unwrap_or_else(|| name.to_string())
    }

    pub(super) fn validation_method_key(
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        span: Span,
    ) -> String {
        symbols
            .map(|symbols| {
                Self::resolver_method_signature_name_for(symbols, ast_key, type_name, span)
            })
            .unwrap_or_else(|| ast_key.to_string())
    }
}
