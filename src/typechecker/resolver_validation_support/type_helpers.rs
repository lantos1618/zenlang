/// Scope for variable types.
#[derive(Debug, Clone)]
struct Scope {
    vars: HashMap<String, VarInfo>,
}

#[derive(Debug, Clone)]
pub(crate) struct VarInfo {
    pub ty: Type,
    pub mutable: bool,
}

impl Scope {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

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

fn type_param_bounds_from_resolver_refs(
    bounds: &[TypeParameterBoundRefMetadata],
) -> HashMap<String, BehaviorBound> {
    bounds
        .iter()
        .map(|bound| {
            (
                bound.type_parameter.clone(),
                BehaviorBound {
                    behavior: bound.behavior.clone(),
                    type_args: bound.type_args.clone(),
                },
            )
        })
        .collect()
}

fn resolver_type_param_bounds(symbol: &crate::resolver::Symbol) -> HashMap<String, BehaviorBound> {
    resolver_type_parameter_metadata(symbol)
        .map(|metadata| type_param_bounds_from_resolver_refs(metadata.bound_refs))
        .unwrap_or_default()
}

fn resolver_type_param_names(symbol: &crate::resolver::Symbol) -> Vec<String> {
    resolver_type_parameter_metadata(symbol)
        .map(|metadata| metadata.names.to_vec())
        .unwrap_or_default()
}

fn resolver_type_parameter_metadata(
    symbol: &crate::resolver::Symbol,
) -> Option<ResolverTypeParameterMetadata<'_>> {
    Some(ResolverTypeParameterMetadata {
        names: symbol.type_parameter_names.as_deref()?,
        bound_refs: symbol.type_parameter_bound_refs.as_deref()?,
    })
}

fn method_signature_key(type_name: &str, method_name: &str) -> String {
    format!("{type_name}.{method_name}")
}

fn behavior_impl_method_signature_key(
    type_name: &str,
    method_name: &str,
    behavior: Option<&str>,
    behavior_type_args: &[AstType],
) -> String {
    let key = method_signature_key(type_name, method_name);
    let Some(behavior) = behavior else {
        return key;
    };
    if behavior_type_args.is_empty() {
        return key;
    }

    format!(
        "{}__{}",
        key,
        behavior_ref_symbol_suffix(behavior, behavior_type_args)
    )
}

fn behavior_ref_symbol_suffix(behavior: &str, type_args: &[AstType]) -> String {
    let mut parts = vec![symbol_key_part(behavior)];
    parts.extend(
        type_args
            .iter()
            .map(AstType::display_name)
            .map(|name| symbol_key_part(&name)),
    );
    parts.join("_")
}

fn symbol_key_part(name: &str) -> String {
    name.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn method_signature_key_parts(name: &str) -> Option<(&str, &str)> {
    name.split_once('.')
}

fn method_signature_receiver_name(name: &str) -> Option<&str> {
    method_signature_key_parts(name).map(|(receiver, _)| receiver)
}

fn method_signature_method_name_for_receiver<'a>(name: &'a str, receiver: &str) -> Option<&'a str> {
    method_signature_key_parts(name)
        .and_then(|(actual_receiver, method)| (actual_receiver == receiver).then_some(method))
}

fn is_method_signature_key(name: &str) -> bool {
    method_signature_key_parts(name).is_some()
}

fn type_param_bound_display(type_param: &ast::TypeParam) -> Option<String> {
    type_param.constraint.as_ref().map(|constraint| {
        if type_param.constraint_type_args.is_empty() {
            constraint.clone()
        } else {
            format!(
                "{}<{}>",
                constraint,
                type_param
                    .constraint_type_args
                    .iter()
                    .map(AstType::display_name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    })
}

fn type_param_name_set(type_params: &[ast::TypeParam]) -> HashSet<String> {
    type_param_names(type_params).into_iter().collect()
}

fn ast_type_references_type_param(
    ast_type: &AstType,
    scoped_type_params: &HashSet<String>,
) -> bool {
    match ast_type {
        AstType::Named(name) => scoped_type_params.contains(name),
        AstType::Generic { type_args, .. } => type_args
            .iter()
            .any(|arg| ast_type_references_type_param(arg, scoped_type_params)),
        AstType::Ptr(inner)
        | AstType::MutPtr(inner)
        | AstType::RawPtr(inner)
        | AstType::Slice(inner)
        | AstType::Array { elem: inner, .. } => {
            ast_type_references_type_param(inner, scoped_type_params)
        }
        AstType::Function { params, ret } => {
            params
                .iter()
                .any(|param| ast_type_references_type_param(param, scoped_type_params))
                || ast_type_references_type_param(ret, scoped_type_params)
        }
        _ => false,
    }
}

fn collect_ast_type_names(ast_type: &AstType, names: &mut HashSet<String>) {
    match ast_type {
        AstType::Named(name) => {
            names.insert(name.clone());
        }
        AstType::Generic { name, type_args } => {
            names.insert(name.clone());
            for type_arg in type_args {
                collect_ast_type_names(type_arg, names);
            }
        }
        AstType::Ptr(inner)
        | AstType::MutPtr(inner)
        | AstType::RawPtr(inner)
        | AstType::Slice(inner)
        | AstType::Array { elem: inner, .. } => collect_ast_type_names(inner, names),
        AstType::Function { params, ret } => {
            for param in params {
                collect_ast_type_names(param, names);
            }
            collect_ast_type_names(ret, names);
        }
        _ => {}
    }
}

fn concrete_self_ast_type(ast_type: &AstType, self_type_name: &str) -> AstType {
    match ast_type {
        AstType::SelfType => AstType::Named(self_type_name.to_string()),
        AstType::Ptr(inner) => {
            AstType::Ptr(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::MutPtr(inner) => {
            AstType::MutPtr(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::RawPtr(inner) => {
            AstType::RawPtr(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::Slice(inner) => {
            AstType::Slice(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(concrete_self_ast_type(elem, self_type_name)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| concrete_self_ast_type(param, self_type_name))
                .collect(),
            ret: Box::new(concrete_self_ast_type(ret, self_type_name)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| concrete_self_ast_type(arg, self_type_name))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}
