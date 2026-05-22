use super::super::*;

#[test]
fn type_impl_generic_method_mutability_tests_live_in_focused_helper() {
    let root =
        read("src/typechecker/tests/resolver_collection/function_method_templates/type_impl_generic_methods/generic_templates.rs");
    let mutability = read(
        "src/typechecker/tests/resolver_collection/function_method_templates/type_impl_generic_methods/param_mutability.rs",
    );
    let module =
        read("src/typechecker/tests/resolver_collection/function_method_templates/type_impl_generic_methods.rs");

    for test_name in [
        "collect_declarations_with_symbols_preserves_type_impl_generic_template_param_mutability_by_position",
        "collect_declarations_with_symbols_ignores_stale_type_impl_generic_template_param_names_for_mutability",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "generic_templates.rs should not own parameter mutability replay test: {test_name}"
        );
        assert!(
            mutability.contains(&format!("fn {test_name}")),
            "type impl generic method parameter mutability replay should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 210,
        "generic_templates.rs should stay focused on generic method template shape metadata"
    );
    assert!(
        module.contains("mod param_mutability;"),
        "type_impl_generic_methods.rs should include the focused parameter mutability module"
    );
}
