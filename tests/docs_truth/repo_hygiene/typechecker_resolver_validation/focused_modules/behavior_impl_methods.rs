use super::*;

#[test]
fn resolver_collection_behavior_impl_method_tests_live_in_focused_modules() {
    let root = read("src/typechecker/tests/resolver_collection/behavior_impl_methods/mod.rs");

    assert!(
        root.lines().count() < 260,
        "resolver collection behavior impl method tests should live in focused modules"
    );
}

#[test]
fn resolver_collection_behavior_impl_restored_generic_templates_live_in_focused_helper() {
    let root = read(
        "src/typechecker/tests/resolver_collection/behavior_impl_methods/restored_signatures.rs",
    );
    let generic_templates = read(
        "src/typechecker/tests/resolver_collection/behavior_impl_methods/restored_signatures/generic_templates.rs",
    );

    for test_name in [
        "collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata",
        "collect_declarations_with_symbols_clears_stale_behavior_impl_generic_method_template_after_key_restore",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "restored_signatures.rs should not own generic restored-template test: {test_name}"
        );
        assert!(
            generic_templates.contains(&format!("fn {test_name}")),
            "generic restored-template test should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 180,
        "restored_signatures.rs should stay focused on non-generic signature restoration"
    );
    assert!(
        root.contains("mod generic_templates;"),
        "restored_signatures.rs should include focused generic-template restoration tests"
    );
}
