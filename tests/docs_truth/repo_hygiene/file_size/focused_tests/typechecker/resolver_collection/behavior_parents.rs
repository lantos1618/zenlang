use super::*;

#[test]
fn resolver_collection_behavior_parent_tests_stay_split_by_restoration_surface() {
    let root = read("src/typechecker/tests/resolver_collection/behavior_parents.rs");
    let duplicate_type_args =
        read("src/typechecker/tests/resolver_collection/behavior_parents/duplicate_type_args.rs");
    let restored_refs =
        read("src/typechecker/tests/resolver_collection/behavior_parents/restored_refs.rs");
    let stale_ast = read("src/typechecker/tests/resolver_collection/behavior_parents/stale_ast.rs");

    assert!(
        root.lines().count() < 80,
        "behavior_parents.rs should only route focused behavior parent collection tests"
    );
    for module in [
        "mod default_synthesis;",
        "mod diagnostics;",
        "mod duplicate_type_args;",
        "mod restored_refs;",
        "mod stale_ast;",
    ] {
        assert!(
            root.contains(module),
            "behavior_parents.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "collect_declarations_with_symbols_uses_resolver_behavior_parent_metadata",
        "collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_parent_metadata",
        "collect_declarations_with_symbols_avoids_false_duplicate_from_restored_parent_type_args",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "behavior_parents.rs should not own concrete test body: {test_name}"
        );
    }
    assert!(
        restored_refs.contains(
            "fn collect_declarations_with_symbols_uses_resolver_behavior_parent_metadata"
        ) && restored_refs.contains(
            "fn collect_declarations_with_symbols_uses_resolver_behavior_parent_and_type_param_metadata"
        ),
        "restored_refs.rs should cover resolver-restored parent refs and type params"
    );
    assert!(
        stale_ast.contains(
            "fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_parent_metadata"
        ),
        "stale_ast.rs should cover missing resolver parent metadata fallback prevention"
    );
    assert!(
        duplicate_type_args.contains(
            "fn collect_declarations_with_symbols_avoids_false_duplicate_from_restored_parent_type_args"
        ),
        "duplicate_type_args.rs should cover restored parent type-arg duplicate keys"
    );
}
