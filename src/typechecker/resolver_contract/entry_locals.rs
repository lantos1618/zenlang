impl TypeChecker {
    fn require_resolver_callable_locals(
        &mut self,
        symbols: &SymbolTable,
        params: &[Param],
        body: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
    ) {
        let mut locals = scope_cursor.new_scope();
        self.require_resolver_parameter_locals(symbols, params, &mut locals);
        self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut locals);
    }

    fn require_resolver_scoped_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
    ) {
        let mut locals = scope_cursor.new_scope();
        self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut locals);
    }
}
