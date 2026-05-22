use super::*;

#[test]
fn resolver_collection_type_metadata_tests_stay_split_by_responsibility() {
    let root = read("src/typechecker/tests/resolver_collection/type_metadata.rs");

    assert!(
        root.lines().count() < 260,
        "resolver collection type metadata tests should stay split by focused responsibility"
    );
}

#[test]
fn resolver_metadata_queue_selection_tests_live_in_focused_helper() {
    let helper = read("src/typechecker/tests/resolver_metadata/impl_and_method_helpers.rs");
    let queue_helper = read("src/typechecker/tests/resolver_metadata/queue_selection.rs");
    let module = read("src/typechecker/tests/resolver_metadata.rs");

    assert!(
        helper.lines().count() < 260,
        "impl_and_method_helpers.rs should stay focused on impl/method metadata helpers"
    );
    assert!(
        !helper.contains("named_queue_selection_prefers_exact_then_front"),
        "queue-selection tests should live in queue_selection.rs"
    );
    assert!(
        queue_helper.contains("resolver_behavior_ref_queue_selection_prefers_exact_then_front"),
        "queue_selection.rs should cover behavior ref queue selection"
    );
    assert!(
        queue_helper.contains("named_queue_selection_can_preserve_front_for_future_match"),
        "queue_selection.rs should cover future-front preservation"
    );
    assert!(
        module.contains("mod queue_selection;"),
        "resolver_metadata.rs should include the focused queue_selection module"
    );
}

#[test]
fn resolver_metadata_impl_and_method_helper_tests_stay_split_by_responsibility() {
    let root = read("src/typechecker/tests/resolver_metadata/impl_and_method_helpers.rs");
    let behavior_collection = read(
        "src/typechecker/tests/resolver_metadata/impl_and_method_helpers/behavior_collection.rs",
    );
    let impl_methods =
        read("src/typechecker/tests/resolver_metadata/impl_and_method_helpers/impl_methods.rs");
    let signatures =
        read("src/typechecker/tests/resolver_metadata/impl_and_method_helpers/signatures.rs");

    assert!(
        root.lines().count() < 80,
        "impl_and_method_helpers.rs should only route focused impl/method helper tests"
    );
    for module in [
        "mod behavior_collection;",
        "mod impl_methods;",
        "mod signatures;",
    ] {
        assert!(
            root.contains(module),
            "impl_and_method_helpers.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains(
            "fn impl_effective_method_name_prefers_resolver_then_ast_then_collected_signature"
        ),
        "impl method selection tests should live in impl_methods.rs"
    );
    assert!(
        behavior_collection
            .contains("fn resolver_backed_behavior_collection_defers_generic_metadata_to_resolver"),
        "behavior_collection.rs should cover resolver-backed behavior collection"
    );
    assert!(
        impl_methods
            .contains("fn effective_behavior_impl_methods_carry_named_declaration_and_method_name"),
        "impl_methods.rs should cover effective impl method metadata"
    );
    assert!(
        signatures.contains("fn resolver_backed_method_signature_requires_resolver_collection"),
        "signatures.rs should cover resolver-backed method signatures"
    );
}
