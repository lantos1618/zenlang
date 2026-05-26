#[derive(Clone, Copy)]
struct BehaviorRefValidation {
    symbol_kind: &'static str,
    name_label: &'static str,
    ref_label: &'static str,
    name_code: DiagnosticCode,
    ref_code: DiagnosticCode,
}

#[derive(Clone, Copy)]
enum BehaviorRefRole {
    Parent,
    Impl,
    Required,
}

#[derive(Clone, Copy)]
enum BehaviorRefCheck {
    Contains,
    List,
}

impl BehaviorRefValidation {
    fn for_role(role: BehaviorRefRole, check: BehaviorRefCheck) -> Self {
        let (symbol_kind, name_label, ref_label) = Self::role_labels(role);
        let (name_code, ref_code) = Self::codes_for(role, check);
        Self {
            symbol_kind,
            name_label,
            ref_label,
            name_code,
            ref_code,
        }
    }

    fn role_labels(role: BehaviorRefRole) -> (&'static str, &'static str, &'static str) {
        match role {
            BehaviorRefRole::Parent => ("behavior", "parents", "parent refs"),
            BehaviorRefRole::Impl => ("type", "behavior impls", "behavior impl refs"),
            BehaviorRefRole::Required => ("type", "behavior requires", "behavior requires refs"),
        }
    }

    fn codes_for(role: BehaviorRefRole, check: BehaviorRefCheck) -> (DiagnosticCode, DiagnosticCode) {
        match (role, check) {
            (BehaviorRefRole::Parent, BehaviorRefCheck::Contains) => (ResolverContractCode::E0235.into(), ResolverContractCode::E0245.into()),
            (BehaviorRefRole::Parent, BehaviorRefCheck::List) => (ResolverContractCode::E0240.into(), ResolverContractCode::E0246.into()),
            (BehaviorRefRole::Impl, BehaviorRefCheck::Contains) => (ResolverContractCode::E0236.into(), ResolverContractCode::E0247.into()),
            (BehaviorRefRole::Impl, BehaviorRefCheck::List) => (ResolverContractCode::E0238.into(), ResolverContractCode::E0248.into()),
            (BehaviorRefRole::Required, BehaviorRefCheck::Contains) => (ResolverContractCode::E0237.into(), ResolverContractCode::E0249.into()),
            (BehaviorRefRole::Required, BehaviorRefCheck::List) => (ResolverContractCode::E0239.into(), ResolverContractCode::E0250.into()),
        }
    }

    fn contains_name_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected to include '{expected}'",
            self.symbol_kind, self.name_label
        )
    }

    fn contains_ref_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected to include '{expected}'",
            self.symbol_kind, self.ref_label
        )
    }

    fn list_name_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected '{expected}'",
            self.symbol_kind, self.name_label
        )
    }

    fn list_ref_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected '{expected}'",
            self.symbol_kind, self.ref_label
        )
    }
}

struct BehaviorRefActual<'a> {
    names: Option<&'a [String]>,
    refs: Option<&'a [BehaviorRefMetadata]>,
}

impl<'a> BehaviorRefActual<'a> {
    fn for_role(symbol: &'a Symbol, role: BehaviorRefRole) -> Self {
        let (names, refs) = Self::metadata_for_role(symbol, role);
        Self { names, refs }
    }

    fn metadata_for_role(
        symbol: &'a Symbol,
        role: BehaviorRefRole,
    ) -> (Option<&'a [String]>, Option<&'a [BehaviorRefMetadata]>) {
        match role {
            BehaviorRefRole::Parent => (
                symbol.behavior_parent_names.as_deref(),
                symbol.behavior_parent_refs.as_deref(),
            ),
            BehaviorRefRole::Impl => (
                symbol.behavior_impl_names.as_deref(),
                symbol.behavior_impl_refs.as_deref(),
            ),
            BehaviorRefRole::Required => (
                symbol.behavior_required_names.as_deref(),
                symbol.behavior_required_refs.as_deref(),
            ),
        }
    }

    fn contains_display(&self, expected: &str) -> bool {
        self.names
            .is_some_and(|names| names.iter().any(|name| name == expected))
    }

    fn contains_metadata(&self, expected: &BehaviorRefMetadata) -> bool {
        self.refs
            .is_some_and(|refs| refs.iter().any(|behavior| behavior == expected))
    }

    fn names_match(&self, expected: &[String]) -> bool {
        behavior_ref_names_match(self.names, expected)
    }

    fn refs_match(&self, expected: &[BehaviorRefMetadata]) -> bool {
        behavior_refs_match(self.refs, expected)
    }
}
