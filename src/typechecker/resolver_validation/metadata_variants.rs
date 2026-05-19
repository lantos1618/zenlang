impl TypeChecker {
    fn validate_resolver_variant_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_variant_names: &[String],
        span: Span,
    ) {
        let validation = VariantNameValidation::resolver_code();
        self.validate_resolver_metadata_list(
            symbol.variant_names.as_deref(),
            expected_variant_names,
            format_variant_names,
            validation.code,
            |actual, expected| validation.message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_variant_payload(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_payload: ExpectedVariantPayloadType,
        span: Span,
    ) {
        let expected = ExpectedVariantPayloadMetadata::from_payload(expected_payload);
        self.validate_resolver_count(
            "variant",
            name,
            symbol.variant_payload_count,
            expected.count,
            CountValidation::variant_payload_resolver_code(),
            span,
        );
        let validation = VariantPayloadValidation::resolver_codes();
        self.validate_resolver_metadata_value(
            symbol.variant_payload_type.as_ref(),
            expected.typed.as_ref(),
            |value| optional_ast_type_display(value, "none"),
            validation.typed_code,
            |actual, expected| validation.typed_message(name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_value(
            symbol.variant_payload_type_name.as_deref(),
            expected.display.as_deref(),
            |value| resolver_metadata_display(value).to_string(),
            validation.display_code,
            |actual, expected| validation.display_message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_variant_owner_name(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_owner_name: &str,
        span: Span,
    ) {
        let validation = VariantOwnerValidation::resolver_code();
        self.validate_resolver_metadata_value(
            symbol.variant_owner_name.as_deref(),
            Some(expected_owner_name),
            |value| resolver_metadata_display(value).to_string(),
            validation.code,
            |actual, expected| validation.message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_variant_visibility(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_is_public: bool,
        span: Span,
    ) {
        self.validate_resolver_visibility(
            "variant",
            name,
            symbol.is_public,
            expected_is_public,
            VisibilityValidation::variant_resolver_code(),
            span,
        );
    }

    fn validate_resolver_variant_absent_other_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_source_metadata(
            symbol,
            "variant",
            name,
            SourceAbsenceValidation::variant_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "variant",
            name,
            ValueSignatureAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "variant",
            name,
            TypeParameterAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "variant",
            name,
            FieldAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "variant",
            name,
            BehaviorAssociationAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "variant",
            name,
            BehaviorDeclarationAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_metadata_entries(
            "variant",
            name,
            &[AbsentMetadataEntry::new(
                symbol.variant_names.is_some(),
                "E0338",
                "variant names",
            )],
            span,
        );
        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "variant",
            name,
            MutabilityAbsenceValidation::variant_resolver_code(),
            span,
        );
    }
}
