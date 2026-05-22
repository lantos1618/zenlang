impl TypeChecker {
    fn require_resolver_parameter_locals(
        &mut self,
        symbols: &SymbolTable,
        params: &[Param],
        locals: &mut ResolverLocalScope,
    ) {
        for param in params {
            self.require_resolver_local_symbol(
                symbols,
                &param.name,
                expected_local_symbol(param.mutable, locals.current_scope_id),
                param.span,
            );
            locals.insert(param.name.clone(), param.mutable);
        }
    }

    fn require_resolver_child_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
        let mut child_locals = scope_cursor.child_scope(locals);
        self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut child_locals);
    }

    fn require_resolver_block_locals(
        &mut self,
        symbols: &SymbolTable,
        statements: &[ast::Statement],
        expr: Option<&Expression>,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
        let mut block_locals = scope_cursor.child_scope(locals);
        for statement in statements {
            self.require_resolver_statement_locals(
                symbols,
                statement,
                scope_cursor,
                &mut block_locals,
            );
        }
        if let Some(expr) = expr {
            self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut block_locals);
        }
    }

    fn require_resolver_closure_locals(
        &mut self,
        symbols: &SymbolTable,
        params: &[Param],
        body: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
        let mut closure_locals = scope_cursor.child_scope(locals);
        self.require_resolver_parameter_locals(symbols, params, &mut closure_locals);
        self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut closure_locals);
    }
}
