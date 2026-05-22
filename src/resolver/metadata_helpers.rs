use crate::ast::{
    behavior_type_args_match_target_params, AstType, BehaviorMethod, EnumVariant, Param,
    StructField, TypeParam,
};

use super::symbol_table::{
    BehaviorMethodTypeMetadata, MethodSignatureMetadata, TypeParameterBoundMetadata,
    TypeParameterBoundRefMetadata, ValueSignatureMetadata,
};

fn resolver_return_type_name(return_type: &Option<AstType>) -> String {
    return_type
        .as_ref()
        .unwrap_or(&AstType::Void)
        .display_name()
}

fn resolver_param_names(params: &[Param]) -> Vec<String> {
    params.iter().map(|param| param.name.clone()).collect()
}

fn resolver_param_type_names(params: &[Param]) -> Vec<String> {
    params.iter().map(|param| param.ty.display_name()).collect()
}

pub(super) fn resolver_value_signature(
    params: &[Param],
    return_type: &Option<AstType>,
    type_params: &[TypeParam],
) -> ValueSignatureMetadata {
    ValueSignatureMetadata {
        parameter_names: resolver_param_names(params),
        parameter_types: params.iter().map(|param| param.ty.clone()).collect(),
        parameter_type_names: resolver_param_type_names(params),
        return_type: return_type.clone().unwrap_or(AstType::Void),
        return_type_name: resolver_return_type_name(return_type),
        type_parameter_count: type_params.len(),
        type_parameter_names: resolver_type_parameter_names(type_params),
        type_parameter_bounds: resolver_type_parameter_bounds(type_params),
        type_parameter_bound_refs: resolver_type_parameter_bound_refs(type_params),
    }
}

pub(super) fn resolver_type_parameter_names(type_params: &[TypeParam]) -> Vec<String> {
    type_params
        .iter()
        .map(|type_param| type_param.name.clone())
        .collect()
}

pub(super) fn resolver_type_parameter_bounds(
    type_params: &[TypeParam],
) -> Vec<TypeParameterBoundMetadata> {
    type_params
        .iter()
        .filter_map(|type_param| {
            type_param_bound_display(type_param)
                .map(|constraint| (type_param.name.clone(), constraint))
        })
        .collect()
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

fn type_param_bound_display(type_param: &TypeParam) -> Option<String> {
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

pub(super) fn behavior_ref_display(behavior: &str, type_args: &[AstType]) -> String {
    if type_args.is_empty() {
        behavior.to_string()
    } else {
        format!(
            "{}<{}>",
            behavior,
            type_args
                .iter()
                .map(AstType::display_name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

pub(super) fn resolver_method_key(type_name: &str, method_name: &str) -> String {
    format!("{type_name}.{method_name}")
}

pub(super) fn resolver_behavior_impl_method_key(
    type_name: &str,
    method_name: &str,
    behavior: &str,
    behavior_type_args: &[AstType],
) -> String {
    if behavior_type_args.is_empty() {
        return resolver_method_key(type_name, method_name);
    }

    format!(
        "{}__{}",
        resolver_method_key(type_name, method_name),
        behavior_ref_symbol_suffix(behavior, behavior_type_args)
    )
}

pub(super) fn resolver_behavior_impl_method_key_with_target_args(
    type_name: &str,
    method_name: &str,
    behavior: &str,
    behavior_type_args: &[AstType],
    target_type_args: &[AstType],
) -> String {
    if target_type_args.is_empty() {
        return resolver_behavior_impl_method_key(
            type_name,
            method_name,
            behavior,
            behavior_type_args,
        );
    }

    if behavior_type_args_match_target_params(behavior_type_args, target_type_args) {
        format!(
            "{}__{}",
            resolver_method_key(type_name, method_name),
            behavior
        )
    } else {
        resolver_behavior_impl_method_key(type_name, method_name, behavior, behavior_type_args)
    }
}

pub(super) fn behavior_ref_symbol_suffix(behavior: &str, type_args: &[AstType]) -> String {
    let mut parts = vec![sanitize_symbol_part(behavior)];
    parts.extend(
        type_args
            .iter()
            .map(AstType::display_name)
            .map(|name| sanitize_symbol_part(&name)),
    );
    parts.join("_")
}

fn sanitize_symbol_part(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    out
}

pub(super) fn resolver_field_types(fields: &[StructField]) -> Vec<(String, AstType, String)> {
    fields
        .iter()
        .map(|field| {
            (
                field.name.clone(),
                field.ty.clone(),
                field.ty.display_name(),
            )
        })
        .collect()
}

pub(super) fn resolver_variant_names(variants: &[EnumVariant]) -> Vec<String> {
    variants
        .iter()
        .map(|variant| variant.name.clone())
        .collect()
}

pub(super) fn resolver_behavior_method_signatures(
    methods: &[BehaviorMethod],
) -> Vec<MethodSignatureMetadata> {
    methods
        .iter()
        .map(|method| {
            (
                method.name.clone(),
                resolver_param_type_names(&method.params),
                resolver_return_type_name(&method.return_type),
            )
        })
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

#[cfg(test)]
mod tests;
