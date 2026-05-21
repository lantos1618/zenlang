use crate::ast::AstType;

pub(in crate::resolver) fn behavior_ref_display(behavior: &str, type_args: &[AstType]) -> String {
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

pub(in crate::resolver) fn resolver_method_key(type_name: &str, method_name: &str) -> String {
    format!("{type_name}.{method_name}")
}

pub(in crate::resolver) fn resolver_behavior_impl_method_key(
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

fn behavior_ref_symbol_suffix(behavior: &str, type_args: &[AstType]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolver_method_key_formats_type_qualified_method_name() {
        assert_eq!(resolver_method_key("Point", "get"), "Point.get");
    }

    #[test]
    fn resolver_behavior_impl_method_key_includes_generic_behavior_specialization() {
        assert_eq!(
            resolver_behavior_impl_method_key("Point", "encode", "Json", &[AstType::Str]),
            "Point.encode__Json_StaticString"
        );
        assert_eq!(
            resolver_behavior_impl_method_key(
                "Point",
                "encode",
                "Json",
                &[AstType::Named("Point".to_string())]
            ),
            "Point.encode__Json_Point"
        );
    }
}
