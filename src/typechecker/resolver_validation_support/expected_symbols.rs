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
