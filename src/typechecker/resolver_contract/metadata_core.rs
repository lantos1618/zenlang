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
}
