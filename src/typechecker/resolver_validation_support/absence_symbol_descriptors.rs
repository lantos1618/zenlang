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
