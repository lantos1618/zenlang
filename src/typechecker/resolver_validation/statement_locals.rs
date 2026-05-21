impl TypeChecker {
    fn require_resolver_statement_locals(
        &mut self,
        symbols: &SymbolTable,
        statement: &ast::Statement,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &mut ResolverLocalScope,
    ) {
        match statement {
            ast::Statement::VarDecl {
                name,
                value,
                mutable,
                span,
                constant,
                ..
            } => {
                self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
                if resolver_var_decl_binds_local(name, *mutable, *constant, locals) {
                    self.require_resolver_var_decl_local(symbols, name, *mutable, *span, locals);
                }
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.require_resolver_expr_locals(symbols, target, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
            }
            ast::Statement::Expression { expr, .. } => {
                self.require_resolver_expr_locals(symbols, expr, scope_cursor, locals);
            }
            ast::Statement::Block { stmts, .. } => {
                self.require_resolver_block_locals(symbols, stmts, None, scope_cursor, locals);
            }
        }
    }

    fn require_resolver_var_decl_local(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        mutable: bool,
        span: Span,
        locals: &mut ResolverLocalScope,
    ) {
        self.require_resolver_local_symbol(
            symbols,
            name,
            expected_local_symbol(mutable, locals.current_scope_id),
            span,
        );
        locals.insert(name.to_string(), mutable);
    }
}
