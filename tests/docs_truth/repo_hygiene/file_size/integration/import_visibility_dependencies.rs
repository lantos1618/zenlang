use super::*;

#[test]
fn import_visibility_dependency_tests_stay_split_by_dependency_shape() {
    let root = read("tests/integration/import_visibility_dependencies.rs");
    let type_impls = read("tests/integration/import_visibility_dependencies/type_impls.rs");
    let generic_functions =
        read("tests/integration/import_visibility_dependencies/generic_functions.rs");
    let signature_types =
        read("tests/integration/import_visibility_dependencies/signature_types.rs");

    assert!(
        root.lines().count() < 60,
        "import_visibility_dependencies.rs should route focused dependency visibility test modules"
    );
    for module in [
        "mod generic_functions;",
        "mod signature_types;",
        "mod type_impls;",
    ] {
        assert!(
            root.contains(module),
            "import_visibility_dependencies.rs should include focused module `{module}`"
        );
    }

    assert!(
        !root.contains("fn imported_type_impl_imported_type_dependencies_are_not_directly_visible"),
        "type-impl dependency visibility test should move out of the root module"
    );
    assert!(
        type_impls
            .contains("fn imported_type_impl_imported_type_dependencies_are_not_directly_visible"),
        "type_impls.rs should keep the type-impl dependency visibility test"
    );

    for generic_function_guard in [
        "fn imported_generic_function_imported_type_dependencies_are_not_directly_visible",
        "fn imported_generic_function_transitive_dependencies_are_not_directly_visible",
    ] {
        assert!(
            !root.contains(generic_function_guard),
            "generic-function dependency visibility test should move out of the root module: {generic_function_guard}"
        );
        assert!(
            generic_functions.contains(generic_function_guard),
            "generic_functions.rs should keep dependency visibility test: {generic_function_guard}"
        );
    }

    assert!(
        !root.contains("fn imported_function_signature_type_dependencies_are_not_directly_visible"),
        "signature type dependency visibility test should move out of the root module"
    );
    assert!(
        signature_types
            .contains("fn imported_function_signature_type_dependencies_are_not_directly_visible"),
        "signature_types.rs should keep the signature type dependency visibility test"
    );
}
