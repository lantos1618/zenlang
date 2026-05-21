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
