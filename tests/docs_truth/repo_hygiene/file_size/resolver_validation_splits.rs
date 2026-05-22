use super::super::*;

#[test]
fn source_dependency_callable_helpers_live_in_focused_module() {
    let root = read("src/typechecker/resolver_validation/imports_source_dependencies.rs");
    let callables =
        read("src/typechecker/resolver_validation/imports_source_dependency_callables.rs");
    let includes = read("src/typechecker/resolver_validation.rs");

    for helper in [
        "fn insert_source_import_type_method_dependencies(",
        "fn insert_source_imported_type_method_dependency(",
        "fn insert_source_function_dependency(",
        "fn insert_source_method_dependency(",
        "fn insert_source_callable_dependency(",
    ] {
        assert!(
            !root.contains(helper),
            "imports_source_dependencies.rs should not own callable helper `{helper}`"
        );
        assert!(
            callables.contains(helper),
            "imported callable source dependency helper should live in focused module: {helper}"
        );
    }

    assert!(
        root.lines().count() < 180,
        "imports_source_dependencies.rs should stay focused on dependency collection and type metadata"
    );
    assert!(
        includes
            .contains("include!(\"resolver_validation/imports_source_dependency_callables.rs\");"),
        "resolver_validation.rs should include focused callable dependency helpers"
    );
}
