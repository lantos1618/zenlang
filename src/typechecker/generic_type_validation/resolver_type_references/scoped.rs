use super::*;

impl TypeChecker {
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
