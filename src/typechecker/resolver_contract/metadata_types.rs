impl TypeChecker {
    fn validate_resolver_type_like_absent_value_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_source_metadata(
            symbol,
            namespace.diagnostic_name(),
            name,
            SourceAbsenceValidation::type_like_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            namespace.diagnostic_name(),
            name,
            ValueSignatureAbsenceValidation::type_like_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            namespace.diagnostic_name(),
            name,
            MutabilityAbsenceValidation::type_like_resolver_code(),
            span,
        );
    }

    fn validate_resolver_fields(
        &mut self,
        symbol: &crate::resolver::Symbol,
        namespace: Namespace,
        name: &str,
        expected_fields: &[ExpectedField],
        span: Span,
    ) {
        let expected = ExpectedFieldMetadata::from_fields(expected_fields);
        self.validate_resolver_count(
            namespace.diagnostic_name(),
            name,
            symbol.field_count,
            expected.count,
            CountValidation::field_resolver_code(),
            span,
        );
        let validation = FieldValidation::resolver_codes();
        let symbol_kind = namespace.diagnostic_name();
        self.validate_resolver_metadata_list(
            symbol.field_types.as_deref(),
            &expected.typed,
            format_field_types,
            validation.typed_code,
            |actual, expected| validation.typed_message(symbol_kind, name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_list(
            symbol.field_type_names.as_deref(),
            &expected.display,
            format_field_type_names,
            validation.display_code,
            |actual, expected| validation.display_message(symbol_kind, name, actual, expected),
            span,
        );
    }

    fn validate_resolver_struct_absent_enum_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_variant_metadata(
            symbol,
            "type",
            name,
            VariantAbsenceValidation::type_like_resolver_codes(),
            span,
        );
    }

    fn validate_resolver_enum_absent_struct_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_field_metadata(
            symbol,
            "type",
            name,
            FieldAbsenceValidation::type_like_resolver_codes(),
            span,
        );
    }

    fn validate_resolver_behavior_methods(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_methods: &[ExpectedBehaviorMethod],
        span: Span,
    ) {
        let expected = ExpectedBehaviorMethodMetadata::from_methods(expected_methods);
        let validation = BehaviorMethodValidation::resolver_codes();
        self.validate_resolver_metadata_list(
            symbol.behavior_method_signatures.as_deref(),
            &expected.signatures,
            format_behavior_method_signatures,
            validation.display_code,
            |actual, expected| validation.display_message(name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_list(
            symbol.behavior_method_types.as_deref(),
            &expected.typed,
            format_behavior_method_types,
            validation.typed_code,
            |actual, expected| validation.typed_message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_behavior_absent_type_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_field_metadata(
            symbol,
            "behavior",
            name,
            FieldAbsenceValidation::behavior_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "behavior",
            name,
            VariantAbsenceValidation::behavior_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "behavior",
            name,
            BehaviorAssociationAbsenceValidation::behavior_resolver_codes(),
            span,
        );
    }

}
