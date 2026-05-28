use super::*;

impl TypeChecker {
    pub(super) fn validate_ast_type_references(&mut self, decl: &Declaration) {
        if let Some(callable) = decl.as_callable() {
            self.validate_ast_callable_type_references(
                callable.type_params,
                callable.params,
                callable.return_type,
                callable.body,
                callable.span,
            );
            return;
        }

        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => {
                let scoped = type_param_name_set(type_params);
                for field in fields {
                    self.validate_generic_type_ref_bounds(&field.ty, &scoped, field.span);
                    if let Some(default) = &field.default {
                        self.validate_generic_expr_type_references(default, &scoped);
                    }
                }
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                let scoped = type_param_name_set(type_params);
                for variant in variants {
                    if let Some(payload) = &variant.payload {
                        self.validate_generic_type_ref_bounds(payload, &scoped, variant.span);
                    }
                }
            }
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => {
                let scoped = type_param_name_set(type_params);
                for method in methods {
                    self.validate_sig_type_refs(
                        &method.params,
                        &method.return_type,
                        &scoped,
                        method.span,
                    );
                    if let Some(default_body) = &method.default_body {
                        self.validate_generic_expr_type_references(default_body, &scoped);
                    }
                }
            }
            Declaration::ImplBlock { methods, .. } => {
                for method in methods {
                    self.validate_ast_type_references(method);
                }
            }
            Declaration::TopLevelExpr { expr, .. } => {
                self.validate_generic_expr_type_references(expr, &HashSet::new());
            }
            _ => {}
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
        self.validate_sig_type_refs(params, return_type, &scoped, return_span);
        self.validate_generic_expr_type_references(body, &scoped);
    }

    fn validate_sig_type_refs(
        &mut self,
        params: &[Param],
        return_type: &Option<AstType>,
        scoped: &HashSet<String>,
        return_span: Span,
    ) {
        for param in params {
            self.validate_generic_type_ref_bounds(&param.ty, scoped, param.span);
        }
        if let Some(return_type) = return_type {
            self.validate_generic_type_ref_bounds(return_type, scoped, return_span);
        }
    }
}
