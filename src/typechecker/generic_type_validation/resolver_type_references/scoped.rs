use super::*;

impl TypeChecker {
    pub(super) fn validate_resolver_named_method_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        method_name: &str,
        body: &Expression,
        span: Span,
    ) {
        let ast_key = Self::method_key(type_name, method_name);
        self.validate_resolver_method_type_references(symbols, &ast_key, type_name, body, span);
    }

    pub(super) fn validate_resolver_scoped_declaration(
        &mut self,
        symbols: Option<&SymbolTable>,
        namespace: Namespace,
        name: &str,
        span: Span,
        scoped_type_params: impl FnOnce(&Self, &str) -> Option<HashSet<String>>,
        validate: impl FnOnce(&mut Self, &str, &HashSet<String>),
    ) {
        let Some((restored_name, scoped)) =
            self.resolver_scoped_symbol(symbols, namespace, name, span, scoped_type_params)
        else {
            return;
        };
        validate(self, &restored_name, &scoped);
    }
}
