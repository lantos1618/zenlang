use super::*;

#[test]
fn resolver_collection_generic_function_template_tests_stay_split_by_responsibility() {
    let root = read(
        "src/typechecker/tests/resolver_collection/function_method_templates/generic_functions.rs",
    );
    let integrity =
        read("src/typechecker/tests/resolver_collection/function_method_templates/generic_functions/integrity.rs");

    assert!(
        root.lines().count() < 180,
        "generic_functions.rs should stay focused on resolver-backed generic function metadata"
    );
    assert!(
        root.contains("mod integrity;"),
        "generic_functions.rs should include the focused integrity module"
    );
    assert!(
        !root.contains("collect_declarations_with_symbols_preserves_generic_template_param_mutability_by_position"),
        "generic function template integrity tests should live in integrity.rs"
    );
    assert!(
        integrity.contains("collect_declarations_with_symbols_uses_resolver_generic_function_template_return_presence"),
        "integrity.rs should cover resolver-backed return presence"
    );
    assert!(
        integrity.contains("collect_declarations_with_symbols_ignores_stale_generic_template_param_names_for_mutability"),
        "integrity.rs should cover positional mutability restoration"
    );
}

#[test]
fn resolver_collection_generic_method_template_tests_stay_split_by_responsibility() {
    let root = read(
        "src/typechecker/tests/resolver_collection/function_method_templates/generic_methods.rs",
    );
    let signature_metadata = read(
        "src/typechecker/tests/resolver_collection/function_method_templates/generic_methods/signature_metadata.rs",
    );
    let integrity =
        read("src/typechecker/tests/resolver_collection/function_method_templates/generic_methods/integrity.rs");

    assert!(
        root.lines().count() < 160,
        "generic_methods.rs should stay focused on resolver-backed generic method metadata"
    );
    for module in ["mod integrity;", "mod signature_metadata;"] {
        assert!(
            root.contains(module),
            "generic_methods.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("collect_declarations_with_symbols_preserves_generic_method_template_param_mutability_by_position"),
        "generic method template signature shape tests should live in signature_metadata.rs"
    );
    assert!(
        signature_metadata.contains("collect_declarations_with_symbols_uses_resolver_generic_method_template_return_presence"),
        "signature_metadata.rs should cover resolver-backed return presence"
    );
    assert!(
        signature_metadata.contains("collect_declarations_with_symbols_ignores_stale_generic_method_template_param_names_for_mutability"),
        "signature_metadata.rs should cover positional mutability restoration"
    );
    assert!(
        integrity.contains("collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_method_template"),
        "integrity.rs should keep stale-AST fallback coverage"
    );
}
