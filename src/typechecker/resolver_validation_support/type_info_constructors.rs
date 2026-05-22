fn type_param_bounds(type_params: &[ast::TypeParam]) -> HashMap<String, BehaviorBound> {
    type_params
        .iter()
        .filter_map(|param| {
            param.constraint.as_ref().map(|bound| {
                (
                    param.name.clone(),
                    BehaviorBound {
                        behavior: bound.clone(),
                        type_args: param.constraint_type_args.clone(),
                    },
                )
            })
        })
        .collect()
}

fn type_param_names(type_params: &[ast::TypeParam]) -> Vec<String> {
    type_params.iter().map(|param| param.name.clone()).collect()
}

fn generic_template_from_type_params(
    type_params: &[ast::TypeParam],
    params: &[Param],
    return_type: &Option<AstType>,
    body: &Expression,
    span: Span,
) -> Option<GenericFunctionTemplate> {
    let collected_type_params = type_param_names(type_params);
    if collected_type_params.is_empty() {
        return None;
    }

    Some(GenericFunctionTemplate::new(
        collected_type_params,
        params.to_vec(),
        return_type.clone(),
        body.clone(),
        span,
    ))
}

fn generic_template_body_stub_from_type_params(
    type_params: &[ast::TypeParam],
    params: &[Param],
    body: &Expression,
    span: Span,
) -> Option<GenericFunctionTemplate> {
    if type_params.is_empty() {
        return None;
    }

    let params = params
        .iter()
        .map(|param| Param {
            name: String::new(),
            ty: AstType::Void,
            mutable: param.mutable,
            span: param.span,
        })
        .collect();
    Some(GenericFunctionTemplate::new(
        Vec::new(),
        params,
        None,
        body.clone(),
        span,
    ))
}

fn func_info_from_ast_signature(
    name: String,
    type_params: &[ast::TypeParam],
    params: &[Param],
    return_type: &Option<AstType>,
) -> FuncInfo {
    FuncInfo {
        name,
        params: params
            .iter()
            .map(|param| (param.name.clone(), param.ty.clone()))
            .collect(),
        return_type: return_type.clone().unwrap_or(AstType::Void),
        type_params: type_param_names(type_params),
        type_param_bounds: type_param_bounds(type_params),
    }
}

fn func_info_from_resolver_signature(
    name: String,
    symbol: &Symbol,
    parameter_names: &[String],
    parameter_types: &[AstType],
    return_type: &AstType,
) -> FuncInfo {
    FuncInfo {
        name,
        params: parameter_names
            .iter()
            .cloned()
            .zip(parameter_types.iter().cloned())
            .collect(),
        return_type: return_type.clone(),
        type_params: resolver_type_param_names(symbol),
        type_param_bounds: resolver_type_param_bounds(symbol),
    }
}

fn struct_info_from_ast_fields(
    name: String,
    type_params: &[ast::TypeParam],
    fields: &[StructField],
) -> StructInfo {
    StructInfo {
        specialization_scope: None,
        name,
        fields: fields
            .iter()
            .map(|field| (field.name.clone(), field.ty.clone()))
            .collect(),
        field_defaults: fields
            .iter()
            .filter_map(|field| {
                field
                    .default
                    .as_ref()
                    .map(|default| (field.name.clone(), default.clone()))
            })
            .collect(),
        type_params: type_param_names(type_params),
        type_param_bounds: type_param_bounds(type_params),
    }
}

fn enum_info_from_ast_variants(
    name: String,
    type_params: &[ast::TypeParam],
    variants: &[EnumVariant],
) -> EnumInfo {
    EnumInfo {
        specialization_scope: None,
        name,
        variants: variants
            .iter()
            .map(|variant| (variant.name.clone(), variant.payload.clone()))
            .collect(),
        type_params: type_param_names(type_params),
        type_param_bounds: type_param_bounds(type_params),
    }
}

fn behavior_info_from_ast_methods(
    name: String,
    type_params: &[ast::TypeParam],
    methods: &[ast::BehaviorMethod],
) -> BehaviorInfo {
    BehaviorInfo {
        name,
        type_params: type_param_names(type_params),
        type_param_bounds: type_param_bounds(type_params),
        methods: methods.to_vec(),
    }
}

fn behavior_info_for_resolver_backed_stub(
    name: String,
    methods: &[ast::BehaviorMethod],
) -> BehaviorInfo {
    BehaviorInfo {
        name,
        type_params: Vec::new(),
        type_param_bounds: HashMap::new(),
        methods: methods.to_vec(),
    }
}

fn struct_info_from_resolver_fields(
    name: String,
    symbol: &Symbol,
    fields: Vec<(String, AstType)>,
    field_defaults: HashMap<String, Expression>,
) -> StructInfo {
    StructInfo {
        specialization_scope: None,
        name,
        fields,
        field_defaults,
        type_params: resolver_type_param_names(symbol),
        type_param_bounds: resolver_type_param_bounds(symbol),
    }
}

fn enum_info_from_resolver_variants(
    name: String,
    symbol: &Symbol,
    variants: Vec<(String, Option<AstType>)>,
) -> EnumInfo {
    EnumInfo {
        specialization_scope: None,
        name,
        variants,
        type_params: resolver_type_param_names(symbol),
        type_param_bounds: resolver_type_param_bounds(symbol),
    }
}

fn behavior_info_from_resolver_methods(
    name: String,
    symbol: &Symbol,
    methods: Vec<ast::BehaviorMethod>,
) -> BehaviorInfo {
    BehaviorInfo {
        name,
        type_params: resolver_type_param_names(symbol),
        type_param_bounds: resolver_type_param_bounds(symbol),
        methods,
    }
}

fn func_info_from_behavior_method(
    name: String,
    params: &[Param],
    return_type: &Option<AstType>,
) -> FuncInfo {
    FuncInfo {
        name,
        params: params
            .iter()
            .map(|param| (param.name.clone(), param.ty.clone()))
            .collect(),
        return_type: return_type.clone().unwrap_or(AstType::Void),
        type_params: Vec::new(),
        type_param_bounds: HashMap::new(),
    }
}
