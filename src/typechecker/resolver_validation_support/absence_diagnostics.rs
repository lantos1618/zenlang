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
