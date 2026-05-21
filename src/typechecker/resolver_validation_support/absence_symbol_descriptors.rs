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
