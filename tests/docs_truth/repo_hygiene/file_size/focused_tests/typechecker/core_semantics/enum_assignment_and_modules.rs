use super::*;

#[test]
fn enum_assignment_and_module_tests_stay_split_by_semantic_surface() {
    let root = read("src/typechecker/tests/core_semantics/enum_assignment_and_modules.rs");
    let assignments =
        read("src/typechecker/tests/core_semantics/enum_assignment_and_modules/assignments.rs");
    let enum_variants =
        read("src/typechecker/tests/core_semantics/enum_assignment_and_modules/enum_variants.rs");
    let field_and_flow =
        read("src/typechecker/tests/core_semantics/enum_assignment_and_modules/field_and_flow.rs");
    let numeric_conversions = read(
        "src/typechecker/tests/core_semantics/enum_assignment_and_modules/numeric_conversions.rs",
    );

    assert!(
        root.lines().count() < 80,
        "enum_assignment_and_modules.rs should only route focused semantic tests"
    );
    for module in [
        "mod assignments;",
        "mod enum_variants;",
        "mod field_and_flow;",
        "mod numeric_conversions;",
    ] {
        assert!(
            root.contains(module),
            "enum_assignment_and_modules.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "fn enum_variant_unknown_variant_is_error",
        "fn assignment_to_immutable_binding_is_error",
        "fn invalid_field_access_is_error",
        "fn implicit_integer_width_conversion_is_error",
    ] {
        assert!(
            !root.contains(test_name),
            "concrete semantic test `{test_name}` should live in a focused child module"
        );
    }
    assert!(
        enum_variants.contains("fn enum_variant_unknown_variant_is_error"),
        "enum_variants.rs should cover unknown enum variants"
    );
    assert!(
        enum_variants.contains("fn enum_variant_payload_type_mismatch_is_error"),
        "enum_variants.rs should cover enum payload type mismatches"
    );
    assert!(
        assignments.contains("fn assignment_to_immutable_binding_is_error"),
        "assignments.rs should cover immutable assignment diagnostics"
    );
    assert!(
        assignments.contains("fn assignment_to_mutable_closure_parameter_is_allowed"),
        "assignments.rs should cover mutable closure parameter assignment"
    );
    assert!(
        assignments.contains("fn assignment_type_mismatch_is_error"),
        "assignments.rs should cover assignment type mismatches"
    );
    assert!(
        field_and_flow.contains("fn invalid_field_access_is_error"),
        "field_and_flow.rs should cover invalid field access diagnostics"
    );
    assert!(
        field_and_flow.contains("fn non_void_function_without_return_is_error"),
        "field_and_flow.rs should cover non-void fallthrough diagnostics"
    );
    assert!(
        numeric_conversions.contains("fn implicit_integer_width_conversion_is_error"),
        "numeric_conversions.rs should cover implicit integer conversion rejection"
    );
    assert!(
        numeric_conversions.contains("fn implicit_float_width_conversion_is_error"),
        "numeric_conversions.rs should cover implicit float conversion rejection"
    );
}
