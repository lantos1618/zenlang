#[derive(Clone, Copy)]
struct TypeParameterAbsenceValidation {
    count_code: &'static str,
    name_code: &'static str,
    bound_code: &'static str,
    bound_ref_code: &'static str,
}

impl TypeParameterAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            count_code: "E0269",
            name_code: "E0348",
            bound_code: "E0270",
            bound_ref_code: "E0373",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            count_code: "E0285",
            name_code: "E0349",
            bound_code: "E0286",
            bound_ref_code: "E0364",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            count_code: "E0253",
            name_code: "E0350",
            bound_code: "E0254",
            bound_ref_code: "E0382",
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            count_code: "E0334",
            name_code: "E0351",
            bound_code: "E0335",
            bound_ref_code: "E0391",
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
    parameter_count_code: &'static str,
    parameter_name_code: &'static str,
    parameter_type_name_code: &'static str,
    parameter_type_code: &'static str,
    return_type_code: &'static str,
    typed_return_type_code: &'static str,
}

impl ValueSignatureAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            parameter_count_code: "E0265",
            parameter_name_code: "E0267",
            parameter_type_name_code: "E0268",
            parameter_type_code: "E0371",
            return_type_code: "E0266",
            typed_return_type_code: "E0372",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            parameter_count_code: "E0281",
            parameter_name_code: "E0283",
            parameter_type_name_code: "E0284",
            parameter_type_code: "E0362",
            return_type_code: "E0282",
            typed_return_type_code: "E0363",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            parameter_count_code: "E0249",
            parameter_name_code: "E0251",
            parameter_type_name_code: "E0252",
            parameter_type_code: "E0380",
            return_type_code: "E0250",
            typed_return_type_code: "E0381",
        }
    }

    fn type_like_resolver_codes() -> Self {
        Self {
            parameter_count_code: "E0310",
            parameter_name_code: "E0312",
            parameter_type_name_code: "E0313",
            parameter_type_code: "E0360",
            return_type_code: "E0311",
            typed_return_type_code: "E0361",
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            parameter_count_code: "E0330",
            parameter_name_code: "E0332",
            parameter_type_name_code: "E0333",
            parameter_type_code: "E0389",
            return_type_code: "E0331",
            typed_return_type_code: "E0390",
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
