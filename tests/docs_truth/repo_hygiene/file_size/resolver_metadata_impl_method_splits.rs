use super::super::*;

#[test]
fn resolver_backed_collection_gates_live_in_focused_helper() {
    let root = read("src/typechecker/tests/resolver_metadata/impl_and_method_helpers.rs");
    let resolver_backed =
        read("src/typechecker/tests/resolver_metadata/impl_and_method_helpers/resolver_backed_collection.rs");

    for test_name in [
        "resolver_backed_impl_method_key_requires_resolver_collection",
        "resolver_backed_method_signature_requires_resolver_collection",
        "behavior_default_synthesis_skip_requires_resolver_collection_and_missing_impl_ref",
        "resolver_backed_behavior_collection_defers_generic_metadata_to_resolver",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "impl_and_method_helpers.rs should not own resolver-backed collection gate test: {test_name}"
        );
        assert!(
            resolver_backed.contains(&format!("fn {test_name}")),
            "resolver-backed collection gate tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 170,
        "impl_and_method_helpers.rs should stay focused on effective method names and signatures"
    );
    assert!(
        root.contains("mod resolver_backed_collection;"),
        "impl_and_method_helpers.rs should include the focused resolver_backed_collection module"
    );
}
