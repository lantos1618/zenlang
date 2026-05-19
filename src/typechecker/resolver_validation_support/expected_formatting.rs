fn visibility_name(is_public: bool) -> &'static str {
    if is_public {
        "public"
    } else {
        "private"
    }
}

fn mutability_name(is_mutable: Option<bool>) -> &'static str {
    match is_mutable {
        Some(true) => "mutable",
        Some(false) => "immutable",
        None => "unknown",
    }
}

fn resolver_count_display(count: Option<usize>) -> String {
    count
        .map(|count| count.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn resolver_metadata_display(value: Option<&str>) -> &str {
    value.unwrap_or("unknown")
}

fn resolver_ast_type_metadata_display(value: Option<&AstType>) -> String {
    optional_ast_type_display(value, "unknown")
}

fn optional_ast_type_display(value: Option<&AstType>, missing: &str) -> String {
    value
        .map(AstType::display_name)
        .unwrap_or_else(|| missing.to_string())
}

fn format_type_parameter_names(names: Option<&[String]>) -> String {
    format_resolver_string_list(names)
}

fn format_type_parameter_bounds(bounds: Option<&[TypeParameterBoundMetadata]>) -> String {
    format_resolver_display_list(bounds, |(name, behavior)| format!("{name}: {behavior}"))
}

fn format_type_parameter_bound_refs(bounds: Option<&[TypeParameterBoundRefMetadata]>) -> String {
    format_resolver_display_list(bounds, |bound| {
        format!(
            "{}: {}",
            bound.type_parameter,
            behavior_ref_display(&bound.behavior, &bound.type_args)
        )
    })
}

fn format_parameter_type_names(names: Option<&[String]>) -> String {
    format_resolver_string_list(names)
}

fn format_ast_type_list(types: Option<&[AstType]>) -> String {
    format_resolver_display_list(types, AstType::display_name)
}

fn format_parameter_names(names: Option<&[String]>) -> String {
    format_resolver_string_list(names)
}

fn format_field_types(fields: Option<&[(String, AstType)]>) -> String {
    format_resolver_named_list(fields, AstType::display_name)
}

fn format_field_type_names(fields: Option<&[(String, String)]>) -> String {
    format_resolver_named_list(fields, String::clone)
}

fn format_variant_names(variants: Option<&[String]>) -> String {
    format_resolver_string_list(variants)
}

fn format_resolver_string_list(values: Option<&[String]>) -> String {
    format_resolver_display_list(values, String::clone)
}

fn format_resolver_display_list<T>(
    values: Option<&[T]>,
    display_value: impl Fn(&T) -> String,
) -> String {
    values
        .map(|values| format!("({})", join_resolver_display_values(values, display_value)))
        .unwrap_or_else(|| "unknown".to_string())
}

fn join_resolver_strings(values: &[String]) -> String {
    values.join(", ")
}

fn join_resolver_display_values<T>(values: &[T], display_value: impl Fn(&T) -> String) -> String {
    let entries = values.iter().map(display_value).collect::<Vec<_>>();
    join_resolver_strings(&entries)
}

fn format_resolver_named_list<T>(
    values: Option<&[(String, T)]>,
    display_value: impl Fn(&T) -> String,
) -> String {
    values
        .map(|values| {
            let entries = values
                .iter()
                .map(|(name, value)| format!("{name}: {}", display_value(value)))
                .collect::<Vec<_>>();
            format!("({})", join_resolver_strings(&entries))
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn format_behavior_method_signatures(methods: Option<&[MethodSignatureMetadata]>) -> String {
    format_resolver_display_list(methods, |(name, params, return_type)| {
        format!("{name}({}) {return_type}", params.join(", "))
    })
}

fn format_behavior_method_types(methods: Option<&[BehaviorMethodTypeMetadata]>) -> String {
    format_resolver_display_list(methods, |method| {
        let params = method
            .parameter_types
            .iter()
            .enumerate()
            .map(|(index, ty)| {
                let name = method
                    .parameter_names
                    .get(index)
                    .map(String::as_str)
                    .unwrap_or("_");
                format!("{name}: {}", ty.display_name())
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{}({}) {}",
            method.name,
            params,
            method.return_type.display_name()
        )
    })
}

fn format_behavior_ref_names(parents: Option<&[String]>) -> String {
    format_resolver_nonempty_joined_list(parents, String::clone)
}

fn format_behavior_refs(refs: Option<&[BehaviorRefMetadata]>) -> String {
    format_resolver_nonempty_joined_list(refs, |behavior| {
        behavior_ref_display(&behavior.name, &behavior.type_args)
    })
}

fn format_resolver_nonempty_joined_list<T>(
    values: Option<&[T]>,
    display_value: impl Fn(&T) -> String,
) -> String {
    match values {
        Some(values) if !values.is_empty() => join_resolver_display_values(values, display_value),
        _ => "none".to_string(),
    }
}

fn behavior_ref_names_match(actual: Option<&[String]>, expected: &[String]) -> bool {
    match actual {
        Some(actual) => actual == expected,
        None => expected.is_empty(),
    }
}

fn behavior_refs_match(
    actual: Option<&[BehaviorRefMetadata]>,
    expected: &[BehaviorRefMetadata],
) -> bool {
    match actual {
        Some(actual) => actual == expected,
        None => expected.is_empty(),
    }
}
