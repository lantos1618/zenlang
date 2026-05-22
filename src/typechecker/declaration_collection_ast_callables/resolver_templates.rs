use super::*;

impl TypeChecker {
    pub(super) fn collect_resolver_backed_function_template(
        &mut self,
        name: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        body: &Expression,
        span: Span,
    ) {
        if let Some(template) =
            generic_template_body_stub_from_type_params(type_params, params, body, span)
        {
            self.generic_functions.insert(name.to_string(), template);
        }
    }

    pub(super) fn collect_resolver_backed_method_template(
        &mut self,
        type_name: &str,
        method_name: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        body: &Expression,
        span: Span,
    ) {
        if let Some(template) =
            generic_template_body_stub_from_type_params(type_params, params, body, span)
        {
            self.generic_methods
                .insert(Self::method_key(type_name, method_name), template);
        }
    }
}
