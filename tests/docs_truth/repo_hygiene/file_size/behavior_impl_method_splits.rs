use super::super::*;

#[test]
fn resolver_behavior_impl_restored_generic_signature_tests_live_in_focused_helper() {
    let root = read(
        "src/typechecker/tests/resolver_collection/behavior_impl_methods/restored_signatures.rs",
    );
    let generic = read(
        "src/typechecker/tests/resolver_collection/behavior_impl_methods/restored_signatures/generic_templates.rs",
    );
    let includes = read("tests/docs_truth/repo_hygiene/file_size.rs");

    for test_name in [
        "collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata",
        "collect_declarations_with_symbols_clears_stale_behavior_impl_generic_method_template_after_key_restore",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "restored_signatures.rs should not own generic restored-signature test: {test_name}"
        );
        assert!(
            generic.contains(&format!("fn {test_name}")),
            "generic restored-signature tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 170,
        "restored_signatures.rs should stay focused on non-generic behavior impl restored signatures"
    );
    assert!(
        root.contains("mod generic_templates;"),
        "restored_signatures.rs should include the focused generic restored-signature tests"
    );
    assert!(
        includes.contains("mod behavior_impl_method_splits;"),
        "file_size.rs should include focused behavior impl method split guards"
    );
}
