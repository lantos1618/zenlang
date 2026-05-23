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

#[test]
fn resolver_phase2_enum_function_payload_tests_live_in_focused_helper() {
    let root = read("tests/resolver_phase2/enum_metadata.rs");
    let function_payloads = read("tests/resolver_phase2/enum_metadata/function_payloads.rs");
    let variant_names = read("tests/resolver_phase2/enum_metadata/variant_names.rs");

    for test_name in [
        "resolver_records_enum_function_type_payloads",
        "resolver_records_generic_enum_function_type_payloads",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "enum_metadata.rs should not own function-type enum payload resolver test: {test_name}"
        );
        assert!(
            function_payloads.contains(&format!("fn {test_name}")),
            "function-type enum payload resolver tests should live in focused helper: {test_name}"
        );
    }
    for test_name in [
        "resolver_records_enum_variant_names",
        "resolver_allows_same_variant_names_in_different_enums",
        "resolver_rejects_duplicate_variant_names_in_same_enum",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "enum_metadata.rs should not own variant-name resolver test: {test_name}"
        );
        assert!(
            variant_names.contains(&format!("fn {test_name}")),
            "variant-name resolver tests should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 150,
        "enum_metadata.rs should stay focused on enum owners, payload counts, and nominal payloads"
    );
    assert!(
        root.contains("#[path = \"enum_metadata/function_payloads.rs\"]"),
        "enum_metadata.rs should include the focused function_payloads module by path"
    );
    assert!(
        root.contains("#[path = \"enum_metadata/variant_names.rs\"]"),
        "enum_metadata.rs should include the focused variant_names module by path"
    );
}
