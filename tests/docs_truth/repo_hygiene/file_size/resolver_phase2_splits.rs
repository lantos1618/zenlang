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
