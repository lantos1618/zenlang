impl TypeChecker {
    fn require_resolver_type_like_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        namespace: Namespace,
        name: &str,
        expected: ExpectedTypeLikeSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let Some(symbol) = symbols.lookup(namespace, name) else {
            self.require_resolver_symbol(symbols, namespace, name, span);
            return None;
        };

        if let Some(expected_is_public) = expected.is_public {
            self.validate_resolver_visibility(
                namespace.diagnostic_name(),
                name,
                symbol.is_public,
                expected_is_public,
                VisibilityValidation::type_like_resolver_code(),
                span,
            );
        }

        self.validate_resolver_type_parameters(
            symbol,
            namespace.diagnostic_name(),
            name,
            &expected.type_params,
            TypeParameterValidation::type_like_resolver_codes(),
            span,
        );

        self.validate_resolver_type_like_absent_value_metadata(symbol, namespace, name, span);

        Some(symbol)
    }

    fn require_resolver_struct_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedStructSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let symbol = self.require_resolver_type_like_symbol(
            symbols,
            Namespace::Type,
            name,
            expected.type_like,
            span,
        )?;

        self.validate_resolver_fields(symbol, Namespace::Type, name, &expected.fields, span);
        self.validate_resolver_struct_absent_enum_metadata(symbol, name, span);

        Some(symbol)
    }

    fn require_resolver_enum_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedEnumSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let symbol = self.require_resolver_type_like_symbol(
            symbols,
            Namespace::Type,
            name,
            expected.type_like,
            span,
        )?;

        self.validate_resolver_variant_names(symbol, name, &expected.variant_names, span);
        self.validate_resolver_enum_absent_struct_metadata(symbol, name, span);

        Some(symbol)
    }

    fn require_resolver_variant_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedVariantSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let Some(symbol) = symbols.lookup_variant(&expected.owner_name, name) else {
            if let Some(symbol) = symbols.lookup(Namespace::Variant, name) {
                self.validate_resolver_variant_owner_name(symbol, name, &expected.owner_name, span);
                return None;
            }
            self.require_resolver_symbol(symbols, Namespace::Variant, name, span);
            return None;
        };

        self.validate_resolver_variant_owner_name(symbol, name, &expected.owner_name, span);
        self.validate_resolver_variant_visibility(symbol, name, expected.is_public, span);
        self.validate_resolver_variant_payload(symbol, name, expected.payload, span);
        self.validate_resolver_variant_absent_other_metadata(symbol, name, span);

        Some(symbol)
    }

    fn require_resolver_behavior_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedBehaviorSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let symbol = self.require_resolver_type_like_symbol(
            symbols,
            Namespace::Behavior,
            name,
            expected.type_like,
            span,
        )?;

        self.validate_resolver_behavior_methods(symbol, name, &expected.methods, span);
        self.validate_resolver_behavior_absent_type_metadata(symbol, name, span);

        Some(symbol)
    }

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

    fn validate_resolver_type_parameters(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        expected: &[ExpectedTypeParameter],
        validation: TypeParameterValidation,
        span: Span,
    ) {
        let expected = ExpectedTypeParameterMetadata::from_parameters(expected);
        self.validate_resolver_count(
            symbol_kind,
            name,
            symbol.type_parameter_count,
            expected.count,
            validation.count_validation(),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.type_parameter_names.as_deref(),
            &expected.names,
            format_type_parameter_names,
            validation.name_code,
            |actual, expected| validation.name_message(symbol_kind, name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.type_parameter_bounds.as_deref(),
            &expected.bounds,
            format_type_parameter_bounds,
            validation.bound_code,
            |actual, expected| validation.bound_message(symbol_kind, name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.type_parameter_bound_refs.as_deref(),
            &expected.bound_refs,
            format_type_parameter_bound_refs,
            validation.bound_ref_code,
            |actual, expected| validation.bound_ref_message(symbol_kind, name, actual, expected),
            span,
        );
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
