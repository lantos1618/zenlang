impl TypeChecker {
    fn require_resolver_pattern_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        pattern: &ast::Pattern,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
        let mut pattern_locals = scope_cursor.child_scope(locals);
        self.require_resolver_pattern_locals(symbols, pattern, scope_cursor, &mut pattern_locals);
        self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut pattern_locals);
    }

    fn require_resolver_pattern_locals(
        &mut self,
        symbols: &SymbolTable,
        pattern: &ast::Pattern,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &mut ResolverLocalScope,
    ) {
        match pattern {
            ast::Pattern::Identifier { name, span } => {
                self.require_resolver_pattern_binding(symbols, name, *span, locals);
            }
            ast::Pattern::Struct { fields, span, .. } => {
                for (name, nested) in fields {
                    if let Some(nested) = nested {
                        self.require_resolver_pattern_locals(symbols, nested, scope_cursor, locals);
                    } else {
                        self.require_resolver_pattern_binding(symbols, name, *span, locals);
                    }
                }
            }
            ast::Pattern::Enum {
                payload: Some(payload),
                ..
            } => {
                self.require_resolver_pattern_locals(symbols, payload, scope_cursor, locals);
            }
            ast::Pattern::Or { patterns, .. } => {
                for pattern in patterns {
                    self.require_resolver_pattern_locals(symbols, pattern, scope_cursor, locals);
                }
            }
            ast::Pattern::Literal { value, .. } => {
                self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
            }
            ast::Pattern::Range { start, end, .. } => {
                self.require_resolver_expr_locals(symbols, start, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, end, scope_cursor, locals);
            }
            ast::Pattern::Wildcard { .. }
            | ast::Pattern::Enum { payload: None, .. }
            | ast::Pattern::BoolTrue { .. }
            | ast::Pattern::BoolFalse { .. } => {}
        }
    }

    fn require_resolver_pattern_binding(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
        locals: &mut ResolverLocalScope,
    ) {
        self.require_resolver_local_symbol(
            symbols,
            name,
            expected_local_symbol(false, locals.current_scope_id),
            span,
        );
        locals.insert(name.to_string(), false);
    }
}
