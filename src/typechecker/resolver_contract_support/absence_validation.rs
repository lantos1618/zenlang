#[derive(Clone, Copy)]
struct FieldAbsenceValidation {
    count_code: DiagnosticCode,
    type_name_code: DiagnosticCode,
    typed_code: DiagnosticCode,
}

impl FieldAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0271.into(),
            type_name_code: ResolverContractCode::E0272.into(),
            typed_code: ResolverContractCode::E0374.into(),
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0287.into(),
            type_name_code: ResolverContractCode::E0288.into(),
            typed_code: ResolverContractCode::E0365.into(),
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0255.into(),
            type_name_code: ResolverContractCode::E0256.into(),
            typed_code: ResolverContractCode::E0383.into(),
        }
    }

    fn type_like_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0319.into(),
            type_name_code: ResolverContractCode::E0320.into(),
            typed_code: ResolverContractCode::E0398.into(),
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0336.into(),
            type_name_code: ResolverContractCode::E0337.into(),
            typed_code: ResolverContractCode::E0392.into(),
        }
    }

    fn behavior_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0321.into(),
            type_name_code: ResolverContractCode::E0322.into(),
            typed_code: ResolverContractCode::E0399.into(),
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0298.into(),
            type_name_code: ResolverContractCode::E0299.into(),
            typed_code: ResolverContractCode::E0403.into(),
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
    names_code: DiagnosticCode,
    owner_code: DiagnosticCode,
    payload_count_code: DiagnosticCode,
    payload_type_name_code: DiagnosticCode,
    payload_type_code: DiagnosticCode,
}

impl VariantAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            names_code: ResolverContractCode::E0273.into(),
            owner_code: ResolverContractCode::E0274.into(),
            payload_count_code: ResolverContractCode::E0275.into(),
            payload_type_name_code: ResolverContractCode::E0276.into(),
            payload_type_code: ResolverContractCode::E0375.into(),
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            names_code: ResolverContractCode::E0289.into(),
            owner_code: ResolverContractCode::E0290.into(),
            payload_count_code: ResolverContractCode::E0291.into(),
            payload_type_name_code: ResolverContractCode::E0292.into(),
            payload_type_code: ResolverContractCode::E0366.into(),
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            names_code: ResolverContractCode::E0257.into(),
            owner_code: ResolverContractCode::E0258.into(),
            payload_count_code: ResolverContractCode::E0259.into(),
            payload_type_name_code: ResolverContractCode::E0260.into(),
            payload_type_code: ResolverContractCode::E0384.into(),
        }
    }

    fn type_like_resolver_codes() -> Self {
        Self {
            names_code: ResolverContractCode::E0315.into(),
            owner_code: ResolverContractCode::E0316.into(),
            payload_count_code: ResolverContractCode::E0317.into(),
            payload_type_name_code: ResolverContractCode::E0318.into(),
            payload_type_code: ResolverContractCode::E0397.into(),
        }
    }

    fn behavior_resolver_codes() -> Self {
        Self {
            names_code: ResolverContractCode::E0323.into(),
            owner_code: ResolverContractCode::E0324.into(),
            payload_count_code: ResolverContractCode::E0325.into(),
            payload_type_name_code: ResolverContractCode::E0326.into(),
            payload_type_code: ResolverContractCode::E0400.into(),
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            names_code: ResolverContractCode::E0300.into(),
            owner_code: ResolverContractCode::E0301.into(),
            payload_count_code: ResolverContractCode::E0302.into(),
            payload_type_name_code: ResolverContractCode::E0303.into(),
            payload_type_code: ResolverContractCode::E0404.into(),
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
