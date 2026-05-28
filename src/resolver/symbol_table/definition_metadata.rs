use super::metadata_helpers::resolver_type_parameter_bound_refs;

fn value_symbol_metadata(signature: ValueSignatureMetadata) -> SymbolMetadata {
    SymbolMetadata {
        parameter_names: Some(signature.parameter_names),
        parameter_types: Some(signature.parameter_types),
        return_type: Some(signature.return_type),
        type_parameter_names: Some(signature.type_parameter_names),
        type_parameter_bound_refs: Some(signature.type_parameter_bound_refs),
        ..SymbolMetadata::default()
    }
}

fn type_like_symbol_metadata(
    type_params: &[crate::ast::TypeParam],
    members: TypeLikeMembers,
) -> SymbolMetadata {
    let (field_types, variant_names) = match members {
        TypeLikeMembers::Fields(fields) => (Some(fields), None),
        TypeLikeMembers::Variants(variants) => (None, Some(variants)),
    };

    SymbolMetadata {
        type_parameter_names: Some(type_param_names(type_params)),
        type_parameter_bound_refs: Some(resolver_type_parameter_bound_refs(type_params)),
        field_types,
        variant_names,
        ..SymbolMetadata::default()
    }
}

fn variant_symbol_metadata(
    owner_name: &str,
    variant_payload_type: Option<AstType>,
) -> SymbolMetadata {
    SymbolMetadata {
        variant_owner_name: Some(owner_name.to_string()),
        variant_payload_type,
        ..SymbolMetadata::default()
    }
}

fn behavior_symbol_metadata(
    type_params: &[crate::ast::TypeParam],
    behavior_method_types: Vec<BehaviorMethodTypeMetadata>,
) -> SymbolMetadata {
    SymbolMetadata {
        type_parameter_names: Some(type_param_names(type_params)),
        type_parameter_bound_refs: Some(resolver_type_parameter_bound_refs(type_params)),
        behavior_method_types: Some(behavior_method_types),
        ..SymbolMetadata::default()
    }
}
