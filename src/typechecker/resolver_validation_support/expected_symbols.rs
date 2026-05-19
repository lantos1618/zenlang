struct ExpectedBehaviorMethod {
    signature: MethodSignatureMetadata,
    metadata: BehaviorMethodTypeMetadata,
}

impl ExpectedBehaviorMethod {
    fn new(method: &ast::BehaviorMethod) -> Self {
        let signature = expected_value_signature_metadata(&method.params, &method.return_type, &[]);
        let parameter_type_names: Vec<_> = signature
            .params
            .iter()
            .map(|param| param.display.clone())
            .collect();
        let parameter_names: Vec<_> = signature
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let parameter_types: Vec<_> = signature
            .params
            .into_iter()
            .map(|param| param.typed)
            .collect();

        Self {
            signature: (
                method.name.clone(),
                parameter_type_names,
                signature.return_type.display,
            ),
            metadata: BehaviorMethodTypeMetadata {
                name: method.name.clone(),
                parameter_names,
                parameter_types,
                return_type: signature.return_type.typed,
            },
        }
    }
}

struct ExpectedBehaviorMethodMetadata {
    signatures: Vec<MethodSignatureMetadata>,
    typed: Vec<BehaviorMethodTypeMetadata>,
}

#[derive(Clone, Copy)]
struct BehaviorMethodValidation {
    display_code: &'static str,
    typed_code: &'static str,
}

impl BehaviorMethodValidation {
    fn resolver_codes() -> Self {
        Self {
            display_code: "E0219",
            typed_code: "E0355",
        }
    }

    fn display_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!("resolver behavior symbol '{name}' has methods '{actual}', expected '{expected}'")
    }

    fn typed_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver behavior symbol '{name}' has typed methods '{actual}', expected '{expected}'"
        )
    }
}

impl ExpectedBehaviorMethodMetadata {
    fn from_methods(methods: &[ExpectedBehaviorMethod]) -> Self {
        Self {
            signatures: methods
                .iter()
                .map(|method| method.signature.clone())
                .collect(),
            typed: methods
                .iter()
                .map(|method| method.metadata.clone())
                .collect(),
        }
    }
}

struct ExpectedBehaviorSymbol {
    type_like: ExpectedTypeLikeSymbol,
    methods: Vec<ExpectedBehaviorMethod>,
}

impl ExpectedBehaviorSymbol {
    fn new(
        type_params: &[ast::TypeParam],
        methods: &[ast::BehaviorMethod],
        is_public: bool,
    ) -> Self {
        Self {
            type_like: ExpectedTypeLikeSymbol::new(type_params, Some(is_public)),
            methods: expected_behavior_method_metadata(methods),
        }
    }
}

struct ExpectedStructSymbol {
    type_like: ExpectedTypeLikeSymbol,
    fields: Vec<ExpectedField>,
}

impl ExpectedStructSymbol {
    fn new(type_params: &[ast::TypeParam], fields: &[StructField], is_public: bool) -> Self {
        Self {
            type_like: ExpectedTypeLikeSymbol::new(type_params, Some(is_public)),
            fields: expected_field_metadata(fields),
        }
    }
}

struct ExpectedEnumSymbol {
    type_like: ExpectedTypeLikeSymbol,
    variant_names: Vec<String>,
}

impl ExpectedEnumSymbol {
    fn new(type_params: &[ast::TypeParam], variants: &[EnumVariant], is_public: bool) -> Self {
        Self {
            type_like: ExpectedTypeLikeSymbol::new(type_params, Some(is_public)),
            variant_names: expected_variant_name_metadata(variants),
        }
    }
}

struct ExpectedVariantSymbol {
    owner_name: String,
    is_public: bool,
    payload: ExpectedVariantPayloadType,
}

impl ExpectedVariantSymbol {
    fn new(owner_name: &str, is_public: bool, payload: &Option<AstType>) -> Self {
        Self {
            owner_name: owner_name.to_string(),
            is_public,
            payload: ExpectedVariantPayloadType::new(payload),
        }
    }
}

struct ExpectedImportSymbol {
    source: String,
    is_public: bool,
}

impl ExpectedImportSymbol {
    fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            is_public: false,
        }
    }
}

struct ExpectedModuleSymbol {
    name: String,
    source: Option<String>,
    is_public: bool,
}

impl ExpectedModuleSymbol {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            source: None,
            is_public: false,
        }
    }
}

struct ExpectedLocalSymbol {
    scope_id: u32,
    is_mutable: bool,
    is_public: bool,
    source: Option<String>,
}

impl ExpectedLocalSymbol {
    fn new(is_mutable: bool, scope_id: u32) -> Self {
        Self {
            scope_id,
            is_mutable,
            is_public: false,
            source: None,
        }
    }
}

struct ExpectedTypeLikeSymbol {
    type_params: Vec<ExpectedTypeParameter>,
    is_public: Option<bool>,
}

impl ExpectedTypeLikeSymbol {
    fn new(type_params: &[ast::TypeParam], is_public: Option<bool>) -> Self {
        Self {
            type_params: expected_type_parameter_metadata(type_params),
            is_public,
        }
    }
}
