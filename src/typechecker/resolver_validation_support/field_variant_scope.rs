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

struct ImportedMethodSignature<'a> {
    name: &'a str,
    type_params: &'a [ast::TypeParam],
    params: &'a [Param],
    return_type: &'a Option<AstType>,
    body: &'a Expression,
    span: Span,
}

impl<'a> ImportedMethodSignature<'a> {
    fn from_function_declaration(name: &'a str, decl: &'a Declaration) -> Option<Self> {
        let Declaration::Function {
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = decl
        else {
            return None;
        };

        Some(Self {
            name,
            type_params,
            params,
            return_type,
            body,
            span: *span,
        })
    }

    fn from_method_declaration(name: &'a str, decl: &'a Declaration) -> Option<Self> {
        let Declaration::Method {
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = decl
        else {
            return None;
        };

        Some(Self {
            name,
            type_params,
            params,
            return_type,
            body,
            span: *span,
        })
    }

    fn func_info(&self, key: String) -> FuncInfo {
        func_info_from_ast_signature(key, self.type_params, self.params, self.return_type)
    }

    fn generic_template(&self) -> Option<GenericFunctionTemplate> {
        generic_template_from_type_params(
            self.type_params,
            self.params,
            self.return_type,
            self.body,
            self.span,
        )
    }
}

#[derive(Debug, Clone)]
struct BehaviorParentRef {
    behavior: String,
    type_args: Vec<AstType>,
    key: String,
}

#[derive(Default)]
struct ResolverScopeCursor {
    next_scope_id: u32,
}

impl ResolverScopeCursor {
    fn new_scope(&mut self) -> ResolverLocalScope {
        self.next_scope_id += 1;
        ResolverLocalScope::new(self.next_scope_id)
    }

    fn child_scope(&mut self, parent: &ResolverLocalScope) -> ResolverLocalScope {
        self.next_scope_id += 1;
        ResolverLocalScope::with_parent(self.next_scope_id, parent)
    }
}

#[derive(Clone)]
struct ResolverLocalScope {
    current_scope_id: u32,
    visible_names: HashMap<String, bool>,
}

impl ResolverLocalScope {
    fn new(current_scope_id: u32) -> Self {
        Self {
            current_scope_id,
            visible_names: HashMap::new(),
        }
    }

    fn with_parent(current_scope_id: u32, parent: &ResolverLocalScope) -> Self {
        Self {
            current_scope_id,
            visible_names: parent.visible_names.clone(),
        }
    }

    fn is_mutable(&self, name: &str) -> bool {
        self.visible_names.get(name).copied().unwrap_or(false)
    }

    fn insert(&mut self, name: String, mutable: bool) {
        self.visible_names.insert(name, mutable);
    }
}
