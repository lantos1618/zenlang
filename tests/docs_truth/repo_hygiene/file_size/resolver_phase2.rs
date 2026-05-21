use super::super::*;

#[test]
fn resolver_phase2_impl_behavior_method_metadata_tests_live_in_focused_helper() {
    let root = read("tests/resolver_phase2/impls.rs");
    let method_metadata = read("tests/resolver_phase2/impls/behavior_method_metadata.rs");

    for test_name in [
        "resolver_records_behavior_impl_methods_as_value_symbols",
        "resolver_records_behavior_impl_function_type_methods",
        "resolver_records_behavior_impl_method_body_locals",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_phase2 impls.rs should not own behavior method metadata test: {test_name}"
        );
        assert!(
            method_metadata.contains(&format!("fn {test_name}")),
            "behavior method metadata test should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 170,
        "resolver_phase2 impls.rs should stay focused on impl-edge and plain impl checks"
    );
    assert!(
        root.contains("mod behavior_method_metadata;"),
        "resolver_phase2 impls.rs should include focused behavior method metadata tests"
    );
}

#[test]
fn resolver_phase2_struct_metadata_tests_live_in_focused_modules() {
    let root = read("tests/resolver_phase2/struct_metadata.rs");
    let declarations = read("tests/resolver_phase2/struct_metadata/declarations.rs");
    let defaults = read("tests/resolver_phase2/struct_metadata/defaults.rs");
    let literals = read("tests/resolver_phase2/struct_metadata/literals.rs");

    assert!(
        root.lines().count() < 60,
        "resolver_phase2 struct_metadata.rs should only route focused struct metadata modules"
    );
    for module in ["mod declarations;", "mod defaults;", "mod literals;"] {
        assert!(
            root.contains(module),
            "resolver_phase2 struct_metadata.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn resolver_rejects_unknown_struct_literal_fields"),
        "struct literal validation tests should live in literals.rs"
    );
    assert!(
        declarations.contains("fn resolver_records_struct_field_types"),
        "declarations.rs should cover struct field metadata"
    );
    assert!(
        defaults.contains("fn resolver_records_struct_field_default_locals"),
        "defaults.rs should cover struct field default local metadata"
    );
    assert!(
        literals.contains("fn resolver_rejects_missing_struct_literal_fields"),
        "literals.rs should cover struct literal field validation"
    );
}
