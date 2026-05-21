struct ExpectedField {
    typed: (String, AstType),
    display: (String, String),
}

impl ExpectedField {
    fn new(name: &str, ty: &AstType) -> Self {
        Self {
            typed: (name.to_string(), ty.clone()),
            display: (name.to_string(), ty.display_name()),
        }
    }
}

struct ExpectedFieldMetadata {
    count: usize,
    typed: Vec<(String, AstType)>,
    display: Vec<(String, String)>,
}

#[derive(Clone, Copy)]
struct FieldValidation {
    display_code: &'static str,
    typed_code: &'static str,
}

impl FieldValidation {
    fn resolver_codes() -> Self {
        Self {
            display_code: "E0217",
            typed_code: "E0358",
        }
    }

    fn display_message(
        self,
        symbol_kind: &str,
        name: &str,
        actual: &str,
        expected: &str,
    ) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has fields '{actual}', expected '{expected}'"
        )
    }

    fn typed_message(self, symbol_kind: &str, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has typed fields '{actual}', expected '{expected}'"
        )
    }
}

impl ExpectedFieldMetadata {
    fn from_fields(fields: &[ExpectedField]) -> Self {
        Self {
            count: fields.len(),
            typed: fields.iter().map(|field| field.typed.clone()).collect(),
            display: fields.iter().map(|field| field.display.clone()).collect(),
        }
    }
}

struct ExpectedVariantPayloadType {
    typed: Option<AstType>,
    display: Option<String>,
}

impl ExpectedVariantPayloadType {
    fn new(payload: &Option<AstType>) -> Self {
        Self {
            typed: payload.clone(),
            display: payload.as_ref().map(AstType::display_name),
        }
    }
}

struct ExpectedVariantPayloadMetadata {
    count: usize,
    typed: Option<AstType>,
    display: Option<String>,
}

#[derive(Clone, Copy)]
struct VariantNameValidation {
    code: &'static str,
}

impl VariantNameValidation {
    fn resolver_code() -> Self {
        Self { code: "E0241" }
    }

    fn message(self, name: &str, actual: &str, expected: &str) -> String {
        format!("resolver type symbol '{name}' has variants '{actual}', expected '{expected}'")
    }
}

#[derive(Clone, Copy)]
struct VariantOwnerValidation {
    code: &'static str,
}

impl VariantOwnerValidation {
    fn resolver_code() -> Self {
        Self { code: "E0242" }
    }

    fn message(self, name: &str, actual: &str, expected: &str) -> String {
        format!("resolver variant symbol '{name}' has owner '{actual}', expected '{expected}'")
    }
}

#[derive(Clone, Copy)]
struct VariantPayloadValidation {
    display_code: &'static str,
    typed_code: &'static str,
}

impl VariantPayloadValidation {
    fn resolver_codes() -> Self {
        Self {
            display_code: "E0218",
            typed_code: "E0359",
        }
    }

    fn display_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver variant symbol '{name}' has payload type '{actual}', expected '{expected}'"
        )
    }

    fn typed_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver variant symbol '{name}' has typed payload type '{actual}', expected '{expected}'"
        )
    }
}

impl ExpectedVariantPayloadMetadata {
    fn from_payload(payload: ExpectedVariantPayloadType) -> Self {
        Self {
            count: usize::from(payload.typed.is_some()),
            typed: payload.typed,
            display: payload.display,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BehaviorParentRef {
    behavior: String,
    type_args: Vec<AstType>,
    key: String,
}
