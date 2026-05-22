use super::*;

#[test]
fn resolver_collection_behavior_impl_metadata_tests_stay_split_by_restoration_surface() {
    let root = read("src/typechecker/tests/resolver_collection/behavior_impls/impl_metadata.rs");
    let restored_refs = read(
        "src/typechecker/tests/resolver_collection/behavior_impls/impl_metadata/restored_refs.rs",
    );
    let stale_ast_fallbacks = read(
        "src/typechecker/tests/resolver_collection/behavior_impls/impl_metadata/stale_ast_fallbacks.rs",
    );

    assert!(
        root.lines().count() < 80,
        "impl_metadata.rs should only route focused behavior impl metadata tests"
    );
    for module in ["mod restored_refs;", "mod stale_ast_fallbacks;"] {
        assert!(
            root.contains(module),
            "impl_metadata.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "fn collect_declarations_with_symbols_uses_resolver_behavior_impl_metadata",
        "fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_impl_metadata",
    ] {
        assert!(
            !root.contains(test_name),
            "concrete behavior impl metadata test `{test_name}` should live in a focused child module"
        );
    }
    assert!(
        restored_refs
            .contains("fn collect_declarations_with_symbols_uses_resolver_behavior_impl_metadata"),
        "restored_refs.rs should cover resolver-restored behavior impl refs"
    );
    assert!(
        restored_refs.contains(
            "fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_and_name_metadata"
        ),
        "restored_refs.rs should cover restored behavior impl target and behavior metadata"
    );
    assert!(
        stale_ast_fallbacks.contains(
            "fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_impl_metadata"
        ),
        "stale_ast_fallbacks.rs should cover missing resolver impl metadata fallbacks"
    );
    assert!(
        stale_ast_fallbacks.contains(
            "fn collect_declarations_with_symbols_does_not_synthesize_stale_impl_defaults_after_target_restore"
        ),
        "stale_ast_fallbacks.rs should cover stale default-method synthesis prevention"
    );
}
