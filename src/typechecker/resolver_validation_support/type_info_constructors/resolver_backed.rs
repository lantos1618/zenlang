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
