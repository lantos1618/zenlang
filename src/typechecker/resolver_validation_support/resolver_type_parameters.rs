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
