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
