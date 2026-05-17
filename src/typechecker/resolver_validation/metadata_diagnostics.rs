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

    fn validate_resolver_source(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: Option<&str>,
        expected: Option<&str>,
        validation: SourceValidation,
        span: Span,
    ) {
        if actual != expected {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_resolver_mutability(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: Option<bool>,
        expected: bool,
        validation: MutabilityValidation,
        span: Span,
    ) {
        if actual != Some(expected) {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_extra_resolver_symbol(
        &mut self,
        symbol_kind: &str,
        name: &str,
        validation: ResolverSymbolPresenceValidation,
        span: Span,
    ) {
        self.validate_resolver_symbol_presence(symbol_kind, name, validation, span);
    }

    fn validate_missing_resolver_symbol(
        &mut self,
        symbol_kind: &str,
        name: &str,
        validation: ResolverSymbolPresenceValidation,
        span: Span,
    ) {
        self.validate_resolver_symbol_presence(symbol_kind, name, validation, span);
    }

    pub(super) fn validate_resolver_symbol_presence(
        &mut self,
        symbol_kind: &str,
        name: &str,
        validation: ResolverSymbolPresenceValidation,
        span: Span,
    ) {
        self.diagnostics.push(Diagnostic::error(
            validation.code,
            validation.message(symbol_kind, name),
            span,
        ));
    }

    fn validate_resolver_absent_metadata_entry(
        &mut self,
        symbol_kind: &str,
        name: &str,
        entry: AbsentMetadataEntry,
        span: Span,
    ) {
        if entry.present {
            self.diagnostics.push(Diagnostic::error(
                entry.code,
                entry.message(symbol_kind, name),
                span,
            ));
        }
    }

    fn validate_resolver_absent_metadata<const N: usize>(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: impl AbsentMetadataValidation<N>,
        span: Span,
    ) {
        let entries = validation.entries(symbol);
        self.validate_resolver_absent_metadata_entries(symbol_kind, name, &entries, span);
    }

    fn validate_resolver_absent_metadata_entries(
        &mut self,
        symbol_kind: &str,
        name: &str,
        entries: &[AbsentMetadataEntry],
        span: Span,
    ) {
        for entry in entries {
            self.validate_resolver_absent_metadata_entry(symbol_kind, name, *entry, span);
        }
    }

    fn validate_resolver_visibility(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: bool,
        expected: bool,
        validation: VisibilityValidation,
        span: Span,
    ) {
        if actual != expected {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_resolver_count(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: Option<usize>,
        expected: usize,
        validation: CountValidation,
        span: Span,
    ) {
        if actual != Some(expected) {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_resolver_metadata_list<T: PartialEq>(
        &mut self,
        actual: Option<&[T]>,
        expected: &[T],
        display: impl Fn(Option<&[T]>) -> String,
        code: &'static str,
        message: impl Fn(&str, &str) -> String,
        span: Span,
    ) {
        if actual != Some(expected) {
            let actual_display = display(actual);
            let expected_display = display(Some(expected));
            self.diagnostics.push(Diagnostic::error(
                code,
                message(&actual_display, &expected_display),
                span,
            ));
        }
    }

    fn validate_resolver_metadata_value<T: PartialEq + ?Sized>(
        &mut self,
        actual: Option<&T>,
        expected: Option<&T>,
        display: impl Fn(Option<&T>) -> String,
        code: &'static str,
        message: impl Fn(&str, &str) -> String,
        span: Span,
    ) {
        if actual != expected {
            let actual_display = display(actual);
            let expected_display = display(expected);
            self.diagnostics.push(Diagnostic::error(
                code,
                message(&actual_display, &expected_display),
                span,
            ));
        }
    }
}
