impl TypeChecker {
    fn require_resolver_module_symbol(
        &mut self,
        symbols: &SymbolTable,
        expected: ExpectedModuleSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Module, &expected.name) else {
            self.require_resolver_symbol(symbols, Namespace::Module, &expected.name, span);
            return;
        };

        self.validate_resolver_visibility(
            "module",
            &expected.name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::module_resolver_code(),
            span,
        );

        self.validate_resolver_source(
            "module",
            &expected.name,
            symbol.import_source.as_deref(),
            expected.source.as_deref(),
            SourceValidation::module_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "module",
            &expected.name,
            ValueSignatureAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "module",
            &expected.name,
            TypeParameterAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "module",
            &expected.name,
            FieldAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "module",
            &expected.name,
            VariantAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "module",
            &expected.name,
            BehaviorAssociationAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "module",
            &expected.name,
            BehaviorDeclarationAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "module",
            &expected.name,
            MutabilityAbsenceValidation::module_resolver_code(),
            span,
        );
    }
}
