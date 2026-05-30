fn type_param_bounds(type_params: &[ast::TypeParam]) -> HashMap<String, BehaviorBound> {
    type_params
        .iter()
        .filter_map(|param| {
            Some((
                param.name.clone(),
                BehaviorBound {
                    behavior: param.constraint.clone()?,
                    type_args: param.constraint_type_args.clone(),
                },
            ))
        })
        .collect()
}

fn type_param_defaults(type_params: &[ast::TypeParam]) -> HashMap<String, AstType> {
    type_params
        .iter()
        .filter_map(|param| Some((param.name.clone(), param.default.clone()?)))
        .collect()
}

fn struct_info_from_ast_fields(
    type_params: &[ast::TypeParam],
    fields: &[StructField],
) -> StructInfo {
    StructInfo {
        specialization_scope: None,
        fields: fields
            .iter()
            .map(|field| (field.name.clone(), field.ty.clone()))
            .collect(),
        field_defaults: fields
            .iter()
            .filter_map(|field| field.default.clone().map(|default| (field.name.clone(), default)))
            .collect(),
        type_params: type_param_names(type_params),
        type_param_bounds: type_param_bounds(type_params),
        type_param_defaults: type_param_defaults(type_params),
    }
}

fn enum_info_from_ast_variants(
    type_params: &[ast::TypeParam],
    variants: &[EnumVariant],
) -> EnumInfo {
    EnumInfo {
        specialization_scope: None,
        variants: variants
            .iter()
            .map(|variant| (variant.name.clone(), variant.payload.clone()))
            .collect(),
        type_params: type_param_names(type_params),
        type_param_bounds: type_param_bounds(type_params),
        type_param_defaults: type_param_defaults(type_params),
    }
}
