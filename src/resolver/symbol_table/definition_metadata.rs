use super::metadata_helpers::{
    resolver_type_parameter_bound_refs, resolver_type_parameter_bounds,
    resolver_type_parameter_names,
};

fn empty_symbol_metadata(import_source: Option<String>) -> SymbolMetadata {
    SymbolMetadata {
        import_source,
        ..SymbolMetadata::default()
    }
}

fn value_symbol_metadata(signature: ValueSignatureMetadata) -> SymbolMetadata {
    let parameter_count = signature.parameter_type_names.len();

    SymbolMetadata {
        parameter_count: Some(parameter_count),
        parameter_names: Some(signature.parameter_names),
        parameter_types: Some(signature.parameter_types),
        parameter_type_names: Some(signature.parameter_type_names),
        return_type: Some(signature.return_type),
        return_type_name: Some(signature.return_type_name),
        type_parameter_count: Some(signature.type_parameter_count),
        type_parameter_names: Some(signature.type_parameter_names),
        type_parameter_bounds: Some(signature.type_parameter_bounds),
        type_parameter_bound_refs: Some(signature.type_parameter_bound_refs),
        ..empty_symbol_metadata(None)
    }
}

fn type_like_symbol_metadata(
    type_params: &[crate::ast::TypeParam],
    members: TypeLikeMembers,
) -> SymbolMetadata {
    let (field_types, field_type_names, variant_names) = match members {
        TypeLikeMembers::Fields(fields) => {
            let typed = fields
                .iter()
                .map(|(name, ty, _)| (name.clone(), ty.clone()))
                .collect();
            let names = fields
                .into_iter()
                .map(|(name, _, type_name)| (name, type_name))
                .collect();
            (Some(typed), Some(names), None)
        }
        TypeLikeMembers::Variants(variants) => (None, None, Some(variants)),
    };

    SymbolMetadata {
        type_parameter_count: Some(type_params.len()),
        type_parameter_names: Some(resolver_type_parameter_names(type_params)),
        type_parameter_bounds: Some(resolver_type_parameter_bounds(type_params)),
        type_parameter_bound_refs: Some(resolver_type_parameter_bound_refs(type_params)),
        field_count: field_type_names.as_ref().map(Vec::len),
        field_types,
        field_type_names,
        variant_names,
        ..empty_symbol_metadata(None)
    }
}

fn variant_symbol_metadata(
    owner_name: &str,
    variant_payload_type: Option<AstType>,
) -> SymbolMetadata {
    let variant_payload_count = usize::from(variant_payload_type.is_some());
    let variant_payload_type_name = variant_payload_type.as_ref().map(AstType::display_name);

    SymbolMetadata {
        variant_owner_name: Some(owner_name.to_string()),
        variant_payload_count: Some(variant_payload_count),
        variant_payload_type,
        variant_payload_type_name,
        ..empty_symbol_metadata(None)
    }
}

fn behavior_symbol_metadata(
    type_params: &[crate::ast::TypeParam],
    behavior_method_signatures: Vec<MethodSignatureMetadata>,
    behavior_method_types: Vec<BehaviorMethodTypeMetadata>,
) -> SymbolMetadata {
    SymbolMetadata {
        type_parameter_count: Some(type_params.len()),
        type_parameter_names: Some(resolver_type_parameter_names(type_params)),
        type_parameter_bounds: Some(resolver_type_parameter_bounds(type_params)),
        type_parameter_bound_refs: Some(resolver_type_parameter_bound_refs(type_params)),
        behavior_method_signatures: Some(behavior_method_signatures),
        behavior_method_types: Some(behavior_method_types),
        ..empty_symbol_metadata(None)
    }
}
