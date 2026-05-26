impl TypeChecker {
    fn validate_resolver_absent_value_signature_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: ValueSignatureAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_type_parameter_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: TypeParameterAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_field_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: FieldAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_variant_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: VariantAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_behavior_association_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: BehaviorAssociationAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_behavior_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: BehaviorDeclarationAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_mutability_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: MutabilityAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_source_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: SourceAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_source(
            symbol_kind,
            name,
            symbol.import_source.as_deref(),
            None,
            validation.source_validation(),
            span,
        );
    }
}
