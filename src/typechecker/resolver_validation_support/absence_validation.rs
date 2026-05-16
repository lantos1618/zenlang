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

trait AbsentMetadataValidation<const N: usize>: Copy {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; N];
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
struct MutabilityValidation {
    code: &'static str,
}

impl MutabilityValidation {
    fn resolver_code() -> Self {
        Self { code: "E0231" }
    }

    fn display(self, actual: Option<bool>, expected: bool) -> (&'static str, &'static str) {
        (mutability_name(actual), mutability_name(Some(expected)))
    }

    fn message(
        self,
        symbol_kind: &str,
        name: &str,
        actual: Option<bool>,
        expected: bool,
    ) -> String {
        let (actual, expected) = self.display(actual, expected);
        format!(
            "resolver {symbol_kind} symbol '{name}' has mutability {actual}, expected {expected}"
        )
    }
}

#[derive(Clone, Copy)]
struct VisibilityValidation {
    code: &'static str,
}

impl VisibilityValidation {
    fn module_resolver_code() -> Self {
        Self { code: "E0229" }
    }

    fn import_resolver_code() -> Self {
        Self { code: "E0245" }
    }

    fn type_like_resolver_code() -> Self {
        Self { code: "E0225" }
    }

    fn variant_resolver_code() -> Self {
        Self { code: "E0226" }
    }

    fn value_resolver_code() -> Self {
        Self { code: "E0224" }
    }

    fn local_resolver_code() -> Self {
        Self { code: "E0247" }
    }

    fn display(self, actual: bool, expected: bool) -> (&'static str, &'static str) {
        (visibility_name(actual), visibility_name(expected))
    }

    fn message(self, symbol_kind: &str, name: &str, actual: bool, expected: bool) -> String {
        let (actual, expected) = self.display(actual, expected);
        format!(
            "resolver {symbol_kind} symbol '{name}' has visibility {actual}, expected {expected}"
        )
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

#[derive(Clone, Copy)]
enum ResolverSymbolPresence {
    Extra,
    Missing,
}

#[derive(Clone, Copy)]
struct ResolverSymbolPresenceValidation {
    code: &'static str,
    presence: ResolverSymbolPresence,
}

impl ResolverSymbolPresenceValidation {
    fn missing_resolver_code() -> Self {
        Self {
            code: "E0210",
            presence: ResolverSymbolPresence::Missing,
        }
    }

    fn missing_local_resolver_code() -> Self {
        Self {
            code: "E0228",
            presence: ResolverSymbolPresence::Missing,
        }
    }

    fn extra_declaration_resolver_code() -> Self {
        Self {
            code: "E0243",
            presence: ResolverSymbolPresence::Extra,
        }
    }

    fn extra_local_resolver_code() -> Self {
        Self {
            code: "E0244",
            presence: ResolverSymbolPresence::Extra,
        }
    }

    fn message(self, symbol_kind: &str, name: &str) -> String {
        let verb = match self.presence {
            ResolverSymbolPresence::Extra => "has extra",
            ResolverSymbolPresence::Missing => "missing",
        };
        format!("resolver symbol table {verb} {symbol_kind} symbol '{name}'")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AbsentMetadataEntry {
    present: bool,
    code: &'static str,
    label: &'static str,
}

impl AbsentMetadataEntry {
    fn new(present: bool, code: &'static str, label: &'static str) -> Self {
        Self {
            present,
            code,
            label,
        }
    }

    fn message(self, symbol_kind: &str, name: &str) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has {} metadata, expected none",
            self.label
        )
    }
}

#[derive(Clone, Copy)]
struct SourceValidation {
    code: &'static str,
    actual_missing: &'static str,
    expected_missing: &'static str,
    quote_expected: bool,
}

impl SourceValidation {
    fn module_resolver_code() -> Self {
        Self {
            code: "E0230",
            actual_missing: "none",
            expected_missing: "none",
            quote_expected: false,
        }
    }

    fn stripped_import_resolver_code() -> Self {
        Self {
            code: "E0246",
            actual_missing: "unknown",
            expected_missing: "a module source",
            quote_expected: false,
        }
    }

    fn import_resolver_code() -> Self {
        Self {
            code: "E0227",
            actual_missing: "unknown",
            expected_missing: "none",
            quote_expected: true,
        }
    }

    fn local_resolver_code() -> Self {
        Self {
            code: "E0248",
            actual_missing: "none",
            expected_missing: "none",
            quote_expected: false,
        }
    }

    fn message(
        self,
        symbol_kind: &str,
        name: &str,
        actual: Option<&str>,
        expected: Option<&str>,
    ) -> String {
        let actual = actual.unwrap_or(self.actual_missing);
        let expected = expected.unwrap_or(self.expected_missing);
        let expected = if self.quote_expected {
            format!("'{expected}'")
        } else {
            expected.to_string()
        };
        format!("resolver {symbol_kind} symbol '{name}' has source '{actual}', expected {expected}")
    }
}

#[derive(Clone, Copy)]
struct CountValidation {
    label: &'static str,
    code: &'static str,
}

impl CountValidation {
    fn value_parameter_resolver_code() -> Self {
        Self {
            label: "parameter count",
            code: "E0211",
        }
    }

    fn field_resolver_code() -> Self {
        Self {
            label: "field count",
            code: "E0214",
        }
    }

    fn variant_payload_resolver_code() -> Self {
        Self {
            label: "payload count",
            code: "E0215",
        }
    }

    fn message(
        self,
        symbol_kind: &str,
        name: &str,
        actual: Option<usize>,
        expected: usize,
    ) -> String {
        let actual = resolver_count_display(actual);
        format!(
            "resolver {symbol_kind} symbol '{name}' has {} {actual}, expected {expected}",
            self.label
        )
    }
}
