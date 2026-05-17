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
