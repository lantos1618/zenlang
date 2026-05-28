use crate::ast::{
    type_param_names, AstType, BehaviorMethod, EnumVariant, Param, StructField, TypeParam,
};

use super::symbol_table::{
    BehaviorMethodTypeMetadata, TypeParameterBoundRefMetadata, ValueSignatureMetadata,
};

fn resolver_param_names(params: &[Param]) -> Vec<String> {
    params.iter().map(|param| param.name.clone()).collect()
}

pub(super) fn resolver_value_signature(
    params: &[Param],
    return_type: &Option<AstType>,
    type_params: &[TypeParam],
) -> ValueSignatureMetadata {
    ValueSignatureMetadata {
        parameter_names: resolver_param_names(params),
        parameter_types: params.iter().map(|param| param.ty.clone()).collect(),
        return_type: return_type.clone().unwrap_or(AstType::Void),
        type_parameter_names: type_param_names(type_params),
        type_parameter_bound_refs: resolver_type_parameter_bound_refs(type_params),
    }
}

pub(super) fn resolver_type_parameter_bound_refs(
    type_params: &[TypeParam],
) -> Vec<TypeParameterBoundRefMetadata> {
    type_params
        .iter()
        .filter_map(|type_param| {
            type_param
                .constraint
                .as_ref()
                .map(|behavior| TypeParameterBoundRefMetadata {
                    type_parameter: type_param.name.clone(),
                    behavior: behavior.clone(),
                    type_args: type_param.constraint_type_args.clone(),
                })
        })
        .collect()
}

pub(super) fn resolver_field_types(fields: &[StructField]) -> Vec<(String, AstType)> {
    fields
        .iter()
        .map(|field| (field.name.clone(), field.ty.clone()))
        .collect()
}

pub(super) fn resolver_variant_names(variants: &[EnumVariant]) -> Vec<String> {
    variants
        .iter()
        .map(|variant| variant.name.clone())
        .collect()
}

pub(super) fn resolver_behavior_method_types(
    methods: &[BehaviorMethod],
) -> Vec<BehaviorMethodTypeMetadata> {
    methods
        .iter()
        .map(|method| BehaviorMethodTypeMetadata {
            name: method.name.clone(),
            parameter_names: resolver_param_names(&method.params),
            parameter_types: method.params.iter().map(|param| param.ty.clone()).collect(),
            return_type: method.return_type.clone().unwrap_or(AstType::Void),
        })
        .collect()
}
