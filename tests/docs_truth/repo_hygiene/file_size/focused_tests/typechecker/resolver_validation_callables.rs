use super::*;

#[test]
fn resolver_validation_callable_tests_stay_split_by_routing_surface() {
    let root = read("src/typechecker/tests/resolver_validation/callables.rs");
    let signature_routing =
        read("src/typechecker/tests/resolver_validation/callables/signature_routing.rs");
    let template_routing =
        read("src/typechecker/tests/resolver_validation/callables/template_routing.rs");
    let resolver_backed =
        read("src/typechecker/tests/resolver_validation/callables/resolver_backed_templates.rs");

    assert!(
        root.lines().count() < 80,
        "callables.rs should only route focused callable resolver-validation tests"
    );
    for module in [
        "mod resolver_backed_templates;",
        "mod signature_routing;",
        "mod template_routing;",
    ] {
        assert!(
            root.contains(module),
            "callables.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "callable_signature_insert_routes_function_and_method_keys",
        "generic_callable_template_mut_routes_function_and_method_keys",
        "callable_template_rekey_routes_function_and_method_keys",
        "resolver_backed_callable_template_collection_defers_signature_metadata_to_resolver",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "callables.rs should not own concrete test body: {test_name}"
        );
    }
    assert!(
        signature_routing.contains("fn callable_signature_insert_routes_function_and_method_keys"),
        "signature_routing.rs should cover concrete callable signature routing"
    );
    assert!(
        template_routing
            .contains("fn generic_callable_template_mut_routes_function_and_method_keys"),
        "template_routing.rs should cover generic callable template mutation"
    );
    assert!(
        template_routing.contains("fn callable_template_rekey_routes_function_and_method_keys"),
        "template_routing.rs should cover generic callable template rekeying"
    );
    assert!(
        resolver_backed.contains(
            "fn resolver_backed_callable_template_collection_defers_signature_metadata_to_resolver",
        ),
        "resolver_backed_templates.rs should cover resolver-backed template stubs"
    );
}
