#[derive(Clone, Copy)]
struct BehaviorAssociationAbsenceValidation {
    impl_name_code: DiagnosticCode,
    impl_ref_code: DiagnosticCode,
    required_name_code: DiagnosticCode,
    required_ref_code: DiagnosticCode,
}

impl BehaviorAssociationAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            impl_name_code: ResolverContractCode::E0279.into(),
            impl_ref_code: ResolverContractCode::E0378.into(),
            required_name_code: ResolverContractCode::E0280.into(),
            required_ref_code: ResolverContractCode::E0379.into(),
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            impl_name_code: ResolverContractCode::E0295.into(),
            impl_ref_code: ResolverContractCode::E0369.into(),
            required_name_code: ResolverContractCode::E0296.into(),
            required_ref_code: ResolverContractCode::E0370.into(),
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            impl_name_code: ResolverContractCode::E0263.into(),
            impl_ref_code: ResolverContractCode::E0387.into(),
            required_name_code: ResolverContractCode::E0264.into(),
            required_ref_code: ResolverContractCode::E0388.into(),
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            impl_name_code: ResolverContractCode::E0341.into(),
            impl_ref_code: ResolverContractCode::E0395.into(),
            required_name_code: ResolverContractCode::E0342.into(),
            required_ref_code: ResolverContractCode::E0396.into(),
        }
    }

    fn behavior_resolver_codes() -> Self {
        Self {
            impl_name_code: ResolverContractCode::E0327.into(),
            impl_ref_code: ResolverContractCode::E0401.into(),
            required_name_code: ResolverContractCode::E0328.into(),
            required_ref_code: ResolverContractCode::E0402.into(),
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            impl_name_code: ResolverContractCode::E0306.into(),
            impl_ref_code: ResolverContractCode::E0407.into(),
            required_name_code: ResolverContractCode::E0307.into(),
            required_ref_code: ResolverContractCode::E0408.into(),
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
    method_signature_code: DiagnosticCode,
    method_type_code: DiagnosticCode,
    parent_name_code: DiagnosticCode,
    parent_ref_code: DiagnosticCode,
}

impl AbsentMetadataValidation<4> for BehaviorDeclarationAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        BehaviorDeclarationAbsenceValidation::entries(self, symbol)
    }
}

impl BehaviorDeclarationAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            method_signature_code: ResolverContractCode::E0277.into(),
            method_type_code: ResolverContractCode::E0376.into(),
            parent_name_code: ResolverContractCode::E0278.into(),
            parent_ref_code: ResolverContractCode::E0377.into(),
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            method_signature_code: ResolverContractCode::E0293.into(),
            method_type_code: ResolverContractCode::E0367.into(),
            parent_name_code: ResolverContractCode::E0294.into(),
            parent_ref_code: ResolverContractCode::E0368.into(),
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            method_signature_code: ResolverContractCode::E0261.into(),
            method_type_code: ResolverContractCode::E0385.into(),
            parent_name_code: ResolverContractCode::E0262.into(),
            parent_ref_code: ResolverContractCode::E0386.into(),
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            method_signature_code: ResolverContractCode::E0339.into(),
            method_type_code: ResolverContractCode::E0393.into(),
            parent_name_code: ResolverContractCode::E0340.into(),
            parent_ref_code: ResolverContractCode::E0394.into(),
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            method_signature_code: ResolverContractCode::E0304.into(),
            method_type_code: ResolverContractCode::E0405.into(),
            parent_name_code: ResolverContractCode::E0305.into(),
            parent_ref_code: ResolverContractCode::E0406.into(),
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
