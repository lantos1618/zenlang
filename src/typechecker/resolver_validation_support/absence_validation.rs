#[derive(Clone, Copy)]
struct FieldAbsenceValidation {
    count_code: &'static str,
    type_name_code: &'static str,
    typed_code: &'static str,
}

impl FieldAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            count_code: "E0271",
            type_name_code: "E0272",
            typed_code: "E0374",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            count_code: "E0287",
            type_name_code: "E0288",
            typed_code: "E0365",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            count_code: "E0255",
            type_name_code: "E0256",
            typed_code: "E0383",
        }
    }

    fn type_like_resolver_codes() -> Self {
        Self {
            count_code: "E0319",
            type_name_code: "E0320",
            typed_code: "E0398",
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            count_code: "E0336",
            type_name_code: "E0337",
            typed_code: "E0392",
        }
    }

    fn behavior_resolver_codes() -> Self {
        Self {
            count_code: "E0321",
            type_name_code: "E0322",
            typed_code: "E0399",
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            count_code: "E0298",
            type_name_code: "E0299",
            typed_code: "E0403",
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 3] {
        [
            AbsentMetadataEntry::new(symbol.field_count.is_some(), self.count_code, "field count"),
            AbsentMetadataEntry::new(
                symbol.field_type_names.is_some(),
                self.type_name_code,
                "field types",
            ),
            AbsentMetadataEntry::new(
                symbol.field_types.is_some(),
                self.typed_code,
                "typed field types",
            ),
        ]
    }
}

impl AbsentMetadataValidation<3> for FieldAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 3] {
        FieldAbsenceValidation::entries(self, symbol)
    }
}

#[derive(Clone, Copy)]
struct VariantAbsenceValidation {
    names_code: &'static str,
    owner_code: &'static str,
    payload_count_code: &'static str,
    payload_type_name_code: &'static str,
    payload_type_code: &'static str,
}

impl VariantAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            names_code: "E0273",
            owner_code: "E0274",
            payload_count_code: "E0275",
            payload_type_name_code: "E0276",
            payload_type_code: "E0375",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            names_code: "E0289",
            owner_code: "E0290",
            payload_count_code: "E0291",
            payload_type_name_code: "E0292",
            payload_type_code: "E0366",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            names_code: "E0257",
            owner_code: "E0258",
            payload_count_code: "E0259",
            payload_type_name_code: "E0260",
            payload_type_code: "E0384",
        }
    }

    fn type_like_resolver_codes() -> Self {
        Self {
            names_code: "E0315",
            owner_code: "E0316",
            payload_count_code: "E0317",
            payload_type_name_code: "E0318",
            payload_type_code: "E0397",
        }
    }

    fn behavior_resolver_codes() -> Self {
        Self {
            names_code: "E0323",
            owner_code: "E0324",
            payload_count_code: "E0325",
            payload_type_name_code: "E0326",
            payload_type_code: "E0400",
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            names_code: "E0300",
            owner_code: "E0301",
            payload_count_code: "E0302",
            payload_type_name_code: "E0303",
            payload_type_code: "E0404",
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 5] {
        [
            AbsentMetadataEntry::new(
                symbol.variant_names.is_some(),
                self.names_code,
                "variant names",
            ),
            AbsentMetadataEntry::new(
                symbol.variant_owner_name.is_some(),
                self.owner_code,
                "variant owner",
            ),
            AbsentMetadataEntry::new(
                symbol.variant_payload_count.is_some(),
                self.payload_count_code,
                "variant payload count",
            ),
            AbsentMetadataEntry::new(
                symbol.variant_payload_type_name.is_some(),
                self.payload_type_name_code,
                "variant payload type",
            ),
            AbsentMetadataEntry::new(
                symbol.variant_payload_type.is_some(),
                self.payload_type_code,
                "typed variant payload type",
            ),
        ]
    }
}

impl AbsentMetadataValidation<5> for VariantAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 5] {
        VariantAbsenceValidation::entries(self, symbol)
    }
}

#[derive(Clone, Copy)]
struct BehaviorAssociationAbsenceValidation {
    impl_name_code: &'static str,
    impl_ref_code: &'static str,
    required_name_code: &'static str,
    required_ref_code: &'static str,
}

impl BehaviorAssociationAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0279",
            impl_ref_code: "E0378",
            required_name_code: "E0280",
            required_ref_code: "E0379",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0295",
            impl_ref_code: "E0369",
            required_name_code: "E0296",
            required_ref_code: "E0370",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0263",
            impl_ref_code: "E0387",
            required_name_code: "E0264",
            required_ref_code: "E0388",
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0341",
            impl_ref_code: "E0395",
            required_name_code: "E0342",
            required_ref_code: "E0396",
        }
    }

    fn behavior_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0327",
            impl_ref_code: "E0401",
            required_name_code: "E0328",
            required_ref_code: "E0402",
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0306",
            impl_ref_code: "E0407",
            required_name_code: "E0307",
            required_ref_code: "E0408",
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        [
            AbsentMetadataEntry::new(
                symbol.behavior_impl_names.is_some(),
                self.impl_name_code,
                "behavior impls",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_impl_refs.is_some(),
                self.impl_ref_code,
                "typed behavior impls",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_required_names.is_some(),
                self.required_name_code,
                "behavior requires",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_required_refs.is_some(),
                self.required_ref_code,
                "typed behavior requires",
            ),
        ]
    }
}

impl AbsentMetadataValidation<4> for BehaviorAssociationAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        BehaviorAssociationAbsenceValidation::entries(self, symbol)
    }
}

#[derive(Clone, Copy)]
struct BehaviorDeclarationAbsenceValidation {
    method_signature_code: &'static str,
    method_type_code: &'static str,
    parent_name_code: &'static str,
    parent_ref_code: &'static str,
}

impl AbsentMetadataValidation<4> for BehaviorDeclarationAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        BehaviorDeclarationAbsenceValidation::entries(self, symbol)
    }
}

impl BehaviorDeclarationAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            method_signature_code: "E0277",
            method_type_code: "E0376",
            parent_name_code: "E0278",
            parent_ref_code: "E0377",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            method_signature_code: "E0293",
            method_type_code: "E0367",
            parent_name_code: "E0294",
            parent_ref_code: "E0368",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            method_signature_code: "E0261",
            method_type_code: "E0385",
            parent_name_code: "E0262",
            parent_ref_code: "E0386",
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            method_signature_code: "E0339",
            method_type_code: "E0393",
            parent_name_code: "E0340",
            parent_ref_code: "E0394",
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            method_signature_code: "E0304",
            method_type_code: "E0405",
            parent_name_code: "E0305",
            parent_ref_code: "E0406",
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        [
            AbsentMetadataEntry::new(
                symbol.behavior_method_signatures.is_some(),
                self.method_signature_code,
                "behavior methods",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_method_types.is_some(),
                self.method_type_code,
                "typed behavior methods",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_parent_names.is_some(),
                self.parent_name_code,
                "behavior parents",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_parent_refs.is_some(),
                self.parent_ref_code,
                "typed behavior parents",
            ),
        ]
    }
}

#[derive(Clone, Copy)]
struct MutabilityAbsenceValidation {
    code: &'static str,
}

impl MutabilityAbsenceValidation {
    fn module_resolver_code() -> Self {
        Self { code: "E0345" }
    }

    fn import_resolver_code() -> Self {
        Self { code: "E0344" }
    }

    fn type_like_resolver_code() -> Self {
        Self { code: "E0314" }
    }

    fn variant_resolver_code() -> Self {
        Self { code: "E0343" }
    }

    fn value_resolver_code() -> Self {
        Self { code: "E0308" }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 1] {
        [AbsentMetadataEntry::new(
            symbol.is_mutable.is_some(),
            self.code,
            "mutability",
        )]
    }
}

impl AbsentMetadataValidation<1> for MutabilityAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 1] {
        MutabilityAbsenceValidation::entries(self, symbol)
    }
}

#[derive(Clone, Copy)]
struct SourceAbsenceValidation {
    code: &'static str,
}

impl SourceAbsenceValidation {
    fn type_like_resolver_code() -> Self {
        Self { code: "E0309" }
    }

    fn variant_resolver_code() -> Self {
        Self { code: "E0329" }
    }

    fn value_resolver_code() -> Self {
        Self { code: "E0297" }
    }

    fn source_validation(self) -> SourceValidation {
        SourceValidation {
            code: self.code,
            actual_missing: "none",
            expected_missing: "none",
            quote_expected: false,
        }
    }
}
