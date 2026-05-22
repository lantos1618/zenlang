use super::*;

#[test]
fn resolver_collection_function_signature_tests_stay_split_by_responsibility() {
    let root = read(
        "src/typechecker/tests/resolver_collection/function_method_templates/function_signatures.rs",
    );
    let resolver_metadata = read(
        "src/typechecker/tests/resolver_collection/function_method_templates/function_signatures/resolver_metadata.rs",
    );
    let stale_ast = read(
        "src/typechecker/tests/resolver_collection/function_method_templates/function_signatures/stale_ast.rs",
    );

    assert!(
        root.lines().count() < 80,
        "function_signatures.rs should route focused resolver-backed function signature tests"
    );
    for module in ["mod resolver_metadata;", "mod stale_ast;"] {
        assert!(
            root.contains(module),
            "function_signatures.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn collect_declarations_with_symbols_uses_resolver_function_type_metadata"),
        "resolver-restored function metadata tests should live in resolver_metadata.rs"
    );
    assert!(
        resolver_metadata
            .contains("fn collect_declarations_with_symbols_uses_resolver_function_type_metadata")
            && resolver_metadata.contains(
                "fn collect_declarations_with_symbols_uses_resolver_function_signature_for_type_refs"
            ),
        "resolver_metadata.rs should cover resolver-restored function signature metadata"
    );
    assert!(
        stale_ast.contains(
            "fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_function_signature"
        ) && stale_ast.contains(
            "fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_function_template"
        ),
        "stale_ast.rs should cover incomplete-signature stale AST fallback prevention"
    );
}

#[test]
fn resolver_collection_bounds_validation_tests_stay_split_by_responsibility() {
    let root = read(
        "src/typechecker/tests/resolver_collection/function_method_templates/bounds_validation.rs",
    );
    let resolver_metadata = read(
        "src/typechecker/tests/resolver_collection/function_method_templates/bounds_validation/resolver_metadata.rs",
    );
    let stale_ast = read(
        "src/typechecker/tests/resolver_collection/function_method_templates/bounds_validation/stale_ast.rs",
    );

    assert!(
        root.lines().count() < 80,
        "bounds_validation.rs should route focused resolver-backed bound validation tests"
    );
    for module in ["mod resolver_metadata;", "mod stale_ast;"] {
        assert!(
            root.contains(module),
            "bounds_validation.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains(
            "fn collect_declarations_with_symbols_uses_resolver_function_bounds_for_validation"
        ),
        "resolver-restored bound metadata tests should live in resolver_metadata.rs"
    );
    assert!(
        resolver_metadata.contains(
            "fn collect_declarations_with_symbols_uses_resolver_function_bounds_for_validation"
        ) && resolver_metadata.contains(
            "fn collect_declarations_with_symbols_uses_resolver_impl_method_bounds_for_validation"
        ),
        "resolver_metadata.rs should cover resolver-restored bound metadata"
    );
    assert!(
        stale_ast.contains(
            "fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_type_bounds"
        ) && stale_ast.contains(
            "fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_bounds"
        ),
        "stale_ast.rs should cover incomplete-bound stale AST fallback prevention"
    );
}
