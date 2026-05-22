use super::super::*;

#[test]
fn resolver_phase2_method_signature_tests_live_in_focused_helper() {
    let root = read("tests/resolver_phase2/core_symbols.rs");
    let methods = read("tests/resolver_phase2/core_symbols/method_signatures.rs");

    for test_name in [
        "resolver_rejects_method_on_unknown_type",
        "resolver_records_method_signatures_as_value_symbols",
        "resolver_records_method_function_type_signatures",
        "resolver_rejects_self_type_outside_method_or_behavior",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "core_symbols.rs should not own resolver method signature test: {test_name}"
        );
        assert!(
            methods.contains(&format!("fn {test_name}")),
            "resolver method signature tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 180,
        "core_symbols.rs should stay focused on core namespace, visibility, type, and import symbols"
    );
    assert!(
        root.contains("#[path = \"core_symbols/method_signatures.rs\"]"),
        "core_symbols.rs should include the focused method signature module by path"
    );
}

#[test]
fn resolver_phase2_non_behavior_impl_tests_live_in_focused_helper() {
    let root = read("tests/resolver_phase2/impls.rs");
    let non_behavior = read("tests/resolver_phase2/impls/non_behavior.rs");

    for test_name in [
        "resolver_accepts_non_behavior_impl_blocks_as_method_symbols",
        "resolver_rejects_duplicate_non_behavior_impl_method_names",
        "resolver_rejects_non_behavior_impl_method_colliding_with_top_level_method",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "impls.rs should not own non-behavior impl test: {test_name}"
        );
        assert!(
            non_behavior.contains(&format!("fn {test_name}")),
            "non-behavior impl tests should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 190,
        "impls.rs should stay focused on behavior impl resolver metadata"
    );
    assert!(
        root.contains("#[path = \"impls/non_behavior.rs\"]"),
        "impls.rs should include the focused non-behavior impl module by path"
    );
}

#[test]
fn resolver_phase2_struct_literal_tests_live_in_focused_helper() {
    let root = read("tests/resolver_phase2/struct_metadata.rs");
    let literals = read("tests/resolver_phase2/struct_metadata/literals.rs");

    for test_name in [
        "resolver_rejects_duplicate_struct_literal_fields",
        "resolver_rejects_unknown_struct_literal_fields",
        "resolver_rejects_missing_struct_literal_fields",
        "resolver_rejects_unknown_struct_literal_types",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "struct_metadata.rs should not own struct-literal resolver test: {test_name}"
        );
        assert!(
            literals.contains(&format!("fn {test_name}")),
            "struct literal resolver tests should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 170,
        "struct_metadata.rs should stay focused on struct declaration metadata"
    );
    assert!(
        root.contains("#[path = \"struct_metadata/literals.rs\"]"),
        "struct_metadata.rs should include the focused struct literal module by path"
    );
}
