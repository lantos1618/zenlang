#[derive(Clone, Copy)]
struct TypeParameterAbsenceValidation {
    count_code: DiagnosticCode,
    name_code: DiagnosticCode,
    bound_code: DiagnosticCode,
    bound_ref_code: DiagnosticCode,
}

impl TypeParameterAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0269.into(),
            name_code: ResolverContractCode::E0348.into(),
            bound_code: ResolverContractCode::E0270.into(),
            bound_ref_code: ResolverContractCode::E0373.into(),
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0285.into(),
            name_code: ResolverContractCode::E0349.into(),
            bound_code: ResolverContractCode::E0286.into(),
            bound_ref_code: ResolverContractCode::E0364.into(),
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0253.into(),
            name_code: ResolverContractCode::E0350.into(),
            bound_code: ResolverContractCode::E0254.into(),
            bound_ref_code: ResolverContractCode::E0382.into(),
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            count_code: ResolverContractCode::E0334.into(),
            name_code: ResolverContractCode::E0351.into(),
            bound_code: ResolverContractCode::E0335.into(),
            bound_ref_code: ResolverContractCode::E0391.into(),
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        [
            AbsentMetadataEntry::new(
                symbol.type_parameter_count.is_some(),
                self.count_code,
                "type parameter count",
            ),
            AbsentMetadataEntry::new(
                symbol.type_parameter_names.is_some(),
                self.name_code,
                "type parameter names",
            ),
            AbsentMetadataEntry::new(
                symbol.type_parameter_bounds.is_some(),
                self.bound_code,
                "type parameter bounds",
            ),
            AbsentMetadataEntry::new(
                symbol.type_parameter_bound_refs.is_some(),
                self.bound_ref_code,
                "typed type parameter bound refs",
            ),
        ]
    }
}

impl AbsentMetadataValidation<4> for TypeParameterAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        TypeParameterAbsenceValidation::entries(self, symbol)
    }
}

#[derive(Clone, Copy)]
struct ValueSignatureAbsenceValidation {
    parameter_count_code: DiagnosticCode,
    parameter_name_code: DiagnosticCode,
    parameter_type_name_code: DiagnosticCode,
    parameter_type_code: DiagnosticCode,
    return_type_code: DiagnosticCode,
    typed_return_type_code: DiagnosticCode,
}

impl ValueSignatureAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            parameter_count_code: ResolverContractCode::E0265.into(),
            parameter_name_code: ResolverContractCode::E0267.into(),
            parameter_type_name_code: ResolverContractCode::E0268.into(),
            parameter_type_code: ResolverContractCode::E0371.into(),
            return_type_code: ResolverContractCode::E0266.into(),
            typed_return_type_code: ResolverContractCode::E0372.into(),
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            parameter_count_code: ResolverContractCode::E0281.into(),
            parameter_name_code: ResolverContractCode::E0283.into(),
            parameter_type_name_code: ResolverContractCode::E0284.into(),
            parameter_type_code: ResolverContractCode::E0362.into(),
            return_type_code: ResolverContractCode::E0282.into(),
            typed_return_type_code: ResolverContractCode::E0363.into(),
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            parameter_count_code: ResolverContractCode::E0249.into(),
            parameter_name_code: ResolverContractCode::E0251.into(),
            parameter_type_name_code: ResolverContractCode::E0252.into(),
            parameter_type_code: ResolverContractCode::E0380.into(),
            return_type_code: ResolverContractCode::E0250.into(),
            typed_return_type_code: ResolverContractCode::E0381.into(),
        }
    }

    fn type_like_resolver_codes() -> Self {
        Self {
            parameter_count_code: ResolverContractCode::E0310.into(),
            parameter_name_code: ResolverContractCode::E0312.into(),
            parameter_type_name_code: ResolverContractCode::E0313.into(),
            parameter_type_code: ResolverContractCode::E0360.into(),
            return_type_code: ResolverContractCode::E0311.into(),
            typed_return_type_code: ResolverContractCode::E0361.into(),
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            parameter_count_code: ResolverContractCode::E0330.into(),
            parameter_name_code: ResolverContractCode::E0332.into(),
            parameter_type_name_code: ResolverContractCode::E0333.into(),
            parameter_type_code: ResolverContractCode::E0389.into(),
            return_type_code: ResolverContractCode::E0331.into(),
            typed_return_type_code: ResolverContractCode::E0390.into(),
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 6] {
        [
            AbsentMetadataEntry::new(
                symbol.parameter_count.is_some(),
                self.parameter_count_code,
                "parameter count",
            ),
            AbsentMetadataEntry::new(
                symbol.parameter_names.is_some(),
                self.parameter_name_code,
                "parameter names",
            ),
            AbsentMetadataEntry::new(
                symbol.parameter_type_names.is_some(),
                self.parameter_type_name_code,
                "parameter types",
            ),
            AbsentMetadataEntry::new(
                symbol.parameter_types.is_some(),
                self.parameter_type_code,
                "typed parameter types",
            ),
            AbsentMetadataEntry::new(
                symbol.return_type_name.is_some(),
                self.return_type_code,
                "return type",
            ),
            AbsentMetadataEntry::new(
                symbol.return_type.is_some(),
                self.typed_return_type_code,
                "typed return type",
            ),
        ]
    }
}

impl AbsentMetadataValidation<6> for ValueSignatureAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 6] {
        ValueSignatureAbsenceValidation::entries(self, symbol)
    }
}
