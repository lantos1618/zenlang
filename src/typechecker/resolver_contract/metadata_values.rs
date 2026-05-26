impl TypeChecker {
    fn require_resolver_value_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected: ExpectedValueSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Value, name) else {
            self.require_resolver_symbol(symbols, Namespace::Value, name, span);
            return;
        };

        self.validate_resolver_visibility(
            "value",
            name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::value_resolver_code(),
            span,
        );

        self.validate_resolver_value_parameters(symbol, name, &expected.signature.params, span);
        self.validate_resolver_value_return_type(
            symbol,
            name,
            &expected.signature.return_type,
            span,
        );

        self.validate_resolver_type_parameters(
            symbol,
            "value",
            name,
            &expected.signature.type_params,
            TypeParameterValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_value_absent_declaration_metadata(symbol, name, span);
    }

    fn validate_resolver_value_parameters(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedParameter],
        span: Span,
    ) {
        let expected = ExpectedParameterMetadata::from_parameters(expected);
        self.validate_resolver_count(
            "value",
            name,
            symbol.parameter_count,
            expected.count,
            CountValidation::value_parameter_resolver_code(),
            span,
        );

        let validation = ValueParameterValidation::resolver_codes();

        self.validate_resolver_metadata_list(
            symbol.parameter_names.as_deref(),
            &expected.names,
            format_parameter_names,
            validation.name_code,
            |actual, expected| validation.name_message(name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.parameter_type_names.as_deref(),
            &expected.display_types,
            format_parameter_type_names,
            validation.display_type_code,
            |actual, expected| validation.display_type_message(name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.parameter_types.as_deref(),
            &expected.typed_types,
            format_ast_type_list,
            validation.typed_type_code,
            |actual, expected| validation.typed_type_message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_value_return_type(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &ExpectedReturnMetadata,
        span: Span,
    ) {
        let validation = ReturnValidation::resolver_codes();

        self.validate_resolver_metadata_value(
            symbol.return_type_name.as_deref(),
            Some(expected.display.as_str()),
            |value| resolver_metadata_display(value).to_string(),
            validation.display_code,
            |actual, expected| validation.display_message(name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_value(
            symbol.return_type.as_ref(),
            Some(&expected.typed),
            resolver_ast_type_metadata_display,
            validation.typed_code,
            |actual, expected| validation.typed_message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_value_absent_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_source_metadata(
            symbol,
            "value",
            name,
            SourceAbsenceValidation::value_resolver_code(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "value",
            name,
            FieldAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "value",
            name,
            VariantAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "value",
            name,
            BehaviorAssociationAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "value",
            name,
            BehaviorDeclarationAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "value",
            name,
            MutabilityAbsenceValidation::value_resolver_code(),
            span,
        );
    }
}
