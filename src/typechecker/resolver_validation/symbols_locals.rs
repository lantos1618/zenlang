impl TypeChecker {
    fn require_resolver_symbol(
        &mut self,
        symbols: &SymbolTable,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) {
        let found = symbols.lookup(namespace, name).is_some()
            || matches!(namespace, Namespace::Type | Namespace::Behavior)
                && symbols.lookup(Namespace::Import, name).is_some();

        if !found {
            self.validate_missing_resolver_symbol(
                namespace.diagnostic_name(),
                name,
                ResolverSymbolPresenceValidation::missing_resolver_code(),
                span,
            );
        }
    }

    fn require_resolver_import_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected: ExpectedImportSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Import, name) else {
            self.require_resolver_symbol(symbols, Namespace::Import, name, span);
            return;
        };

        self.validate_resolver_visibility(
            "import",
            name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::import_resolver_code(),
            span,
        );

        self.validate_resolver_source(
            "import",
            name,
            symbol.import_source.as_deref(),
            Some(expected.source.as_str()),
            SourceValidation::import_resolver_code(),
            span,
        );

        self.validate_resolver_import_absent_declaration_metadata(symbol, name, span);
    }

    fn validate_resolver_import_absent_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "import",
            name,
            ValueSignatureAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "import",
            name,
            TypeParameterAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "import",
            name,
            FieldAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "import",
            name,
            VariantAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "import",
            name,
            BehaviorAssociationAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "import",
            name,
            BehaviorDeclarationAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "import",
            name,
            MutabilityAbsenceValidation::import_resolver_code(),
            span,
        );
    }

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
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
                }
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
            | Expression::CharLiteral { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::LoopControl { .. }
            | Expression::Error { .. } => {}
        }
    }

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

    fn require_resolver_local_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected: ExpectedLocalSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup_in_scope(Namespace::Local, name, expected.scope_id)
        else {
            self.validate_missing_resolver_symbol(
                "local",
                name,
                ResolverSymbolPresenceValidation::missing_local_resolver_code(),
                span,
            );
            return;
        };

        self.validate_resolver_mutability(
            "local",
            name,
            symbol.is_mutable,
            expected.is_mutable,
            MutabilityValidation::resolver_code(),
            span,
        );

        self.validate_resolver_visibility(
            "local",
            name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::local_resolver_code(),
            span,
        );

        self.validate_resolver_source(
            "local",
            name,
            symbol.import_source.as_deref(),
            expected.source.as_deref(),
            SourceValidation::local_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "local",
            name,
            ValueSignatureAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "local",
            name,
            TypeParameterAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "local",
            name,
            FieldAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "local",
            name,
            VariantAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "local",
            name,
            BehaviorAssociationAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "local",
            name,
            BehaviorDeclarationAbsenceValidation::local_resolver_codes(),
            span,
        );
    }

}
