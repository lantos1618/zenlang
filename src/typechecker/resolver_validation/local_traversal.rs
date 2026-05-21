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

    fn require_resolver_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &mut ResolverLocalScope,
    ) {
        match expr {
            Expression::BinaryOp { left, right, .. } => {
                self.require_resolver_expr_locals(symbols, left, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, right, scope_cursor, locals);
            }
            Expression::UnaryOp { operand, .. } => {
                self.require_resolver_expr_locals(symbols, operand, scope_cursor, locals);
            }
            Expression::FunctionCall { args, .. } => {
                for arg in args {
                    self.require_resolver_expr_locals(symbols, arg, scope_cursor, locals);
                }
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.require_resolver_expr_locals(symbols, receiver, scope_cursor, locals);
                for arg in args {
                    self.require_resolver_expr_locals(symbols, arg, scope_cursor, locals);
                }
            }
            Expression::MemberAccess { object, .. } => {
                self.require_resolver_expr_locals(symbols, object, scope_cursor, locals);
            }
            Expression::IndexAccess { object, index, .. } => {
                self.require_resolver_expr_locals(symbols, object, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, index, scope_cursor, locals);
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
                }
            }
            Expression::EnumVariant { payload, .. } => {
                if let Some(payload) = payload {
                    self.require_resolver_expr_locals(symbols, payload, scope_cursor, locals);
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.require_resolver_expr_locals(symbols, element, scope_cursor, locals);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.require_resolver_expr_locals(symbols, scrutinee, scope_cursor, locals);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.require_resolver_pattern_expr_locals(
                            symbols,
                            &arm.pattern,
                            guard,
                            scope_cursor,
                            locals,
                        );
                    }
                    self.require_resolver_pattern_expr_locals(
                        symbols,
                        &arm.pattern,
                        &arm.body,
                        scope_cursor,
                        locals,
                    );
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.require_resolver_expr_locals(symbols, condition, scope_cursor, locals);
                self.require_resolver_child_expr_locals(symbols, body, scope_cursor, locals);
            }
            Expression::Loop { body, .. } => {
                self.require_resolver_child_expr_locals(symbols, body, scope_cursor, locals);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.require_resolver_expr_locals(symbols, condition, scope_cursor, locals);
                self.require_resolver_child_expr_locals(symbols, then_body, scope_cursor, locals);
                if let Some(else_body) = else_body {
                    self.require_resolver_child_expr_locals(
                        symbols,
                        else_body,
                        scope_cursor,
                        locals,
                    );
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                self.require_resolver_block_locals(
                    symbols,
                    statements,
                    expr.as_deref(),
                    scope_cursor,
                    locals,
                );
            }
            Expression::Closure { params, body, .. } => {
                self.require_resolver_closure_locals(symbols, params, body, scope_cursor, locals);
            }
            Expression::Cast { expr, .. } | Expression::Defer { expr, .. } => {
                self.require_resolver_expr_locals(symbols, expr, scope_cursor, locals);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let ast::StringPart::Expr(expr) = part {
                        self.require_resolver_expr_locals(symbols, expr, scope_cursor, locals);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.require_resolver_expr_locals(symbols, start, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, end, scope_cursor, locals);
            }
            Expression::Identifier { .. }
            | Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::LoopControl { .. }
            | Expression::Error { .. } => {}
        }
    }

}
