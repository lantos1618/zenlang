#[derive(Clone, Copy)]
struct MutabilityAbsenceValidation {
    code: DiagnosticCode,
}

impl MutabilityAbsenceValidation {
    fn module_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0345.into() }
    }

    fn import_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0344.into() }
    }

    fn type_like_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0314.into() }
    }

    fn variant_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0343.into() }
    }

    fn value_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0308.into() }
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
    code: DiagnosticCode,
}

impl SourceAbsenceValidation {
    fn type_like_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0309.into() }
    }

    fn variant_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0329.into() }
    }

    fn value_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0297.into() }
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
