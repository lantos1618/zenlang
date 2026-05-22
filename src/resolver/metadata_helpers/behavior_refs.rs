use crate::ast::{behavior_type_args_match_target_params, AstType};

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

pub(in crate::resolver) fn resolver_behavior_impl_method_key_with_target_args(
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

pub(in crate::resolver) fn behavior_ref_symbol_suffix(
    behavior: &str,
    type_args: &[AstType],
) -> String {
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
