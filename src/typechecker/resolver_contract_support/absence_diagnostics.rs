#[derive(Clone, Copy)]
struct MutabilityValidation {
    code: DiagnosticCode,
}

impl MutabilityValidation {
    fn resolver_code() -> Self {
        Self { code: ResolverContractCode::E0231.into() }
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
    code: DiagnosticCode,
}

impl VisibilityValidation {
    fn module_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0229.into() }
    }

    fn import_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0245.into() }
    }

    fn type_like_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0225.into() }
    }

    fn variant_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0226.into() }
    }

    fn value_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0224.into() }
    }

    fn local_resolver_code() -> Self {
        Self { code: ResolverContractCode::E0247.into() }
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
enum ResolverSymbolPresence {
    Extra,
    Missing,
}

#[derive(Clone, Copy)]
struct ResolverSymbolPresenceValidation {
    code: DiagnosticCode,
    presence: ResolverSymbolPresence,
}

impl ResolverSymbolPresenceValidation {
    fn missing_resolver_code() -> Self {
        Self {
            code: ResolverContractCode::E0210.into(),
            presence: ResolverSymbolPresence::Missing,
        }
    }

    fn missing_local_resolver_code() -> Self {
        Self {
            code: ResolverContractCode::E0228.into(),
            presence: ResolverSymbolPresence::Missing,
        }
    }

    fn extra_declaration_resolver_code() -> Self {
        Self {
            code: ResolverContractCode::E0243.into(),
            presence: ResolverSymbolPresence::Extra,
        }
    }

    fn extra_local_resolver_code() -> Self {
        Self {
            code: ResolverContractCode::E0244.into(),
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

#[derive(Clone, Copy)]
struct SourceValidation {
    code: DiagnosticCode,
    actual_missing: &'static str,
    expected_missing: &'static str,
    quote_expected: bool,
}

impl SourceValidation {
    fn module_resolver_code() -> Self {
        Self {
            code: ResolverContractCode::E0230.into(),
            actual_missing: "none",
            expected_missing: "none",
            quote_expected: false,
        }
    }

    fn stripped_import_resolver_code() -> Self {
        Self {
            code: ResolverContractCode::E0246.into(),
            actual_missing: "unknown",
            expected_missing: "a module source",
            quote_expected: false,
        }
    }

    fn import_resolver_code() -> Self {
        Self {
            code: ResolverContractCode::E0227.into(),
            actual_missing: "unknown",
            expected_missing: "none",
            quote_expected: true,
        }
    }

    fn local_resolver_code() -> Self {
        Self {
            code: ResolverContractCode::E0248.into(),
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
    code: DiagnosticCode,
}

impl CountValidation {
    fn value_parameter_resolver_code() -> Self {
        Self {
            label: "parameter count",
            code: ResolverContractCode::E0211.into(),
        }
    }

    fn field_resolver_code() -> Self {
        Self {
            label: "field count",
            code: ResolverContractCode::E0214.into(),
        }
    }

    fn variant_payload_resolver_code() -> Self {
        Self {
            label: "payload count",
            code: ResolverContractCode::E0215.into(),
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
