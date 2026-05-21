use super::*;

#[test]
fn typechecker_resolver_expected_formatting_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let helpers = read("src/typechecker/resolver_validation_support/expected_helpers.rs");
    let formatting = read("src/typechecker/resolver_validation_support/expected_formatting.rs");

    for helper in [
        "visibility_name",
        "mutability_name",
        "resolver_count_display",
        "resolver_metadata_display",
        "resolver_ast_type_metadata_display",
        "optional_ast_type_display",
        "format_type_parameter_names",
        "format_type_parameter_bounds",
        "format_type_parameter_bound_refs",
        "format_parameter_type_names",
        "format_ast_type_list",
        "format_parameter_names",
        "format_field_types",
        "format_field_type_names",
        "format_variant_names",
        "format_resolver_string_list",
        "format_resolver_display_list",
        "join_resolver_strings",
        "join_resolver_display_values",
        "format_resolver_named_list",
        "format_behavior_method_signatures",
        "format_behavior_method_types",
        "format_behavior_ref_names",
        "format_behavior_refs",
        "format_resolver_nonempty_joined_list",
        "behavior_ref_names_match",
        "behavior_refs_match",
    ] {
        assert!(
            !helpers.contains(&format!("fn {helper}")),
            "expected_helpers.rs should not own resolver formatting helper: {helper}"
        );
        assert!(
            formatting.contains(&format!("fn {helper}")),
            "resolver expected formatting should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("include!(\"resolver_validation_support/expected_formatting.rs\");"),
        "resolver validation support should include focused expected formatting helper"
    );
}
