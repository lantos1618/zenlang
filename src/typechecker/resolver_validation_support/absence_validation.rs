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
