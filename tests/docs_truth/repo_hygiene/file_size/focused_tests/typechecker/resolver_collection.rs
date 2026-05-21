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

#[test]
fn resolver_metadata_restoration_tests_stay_split_by_responsibility() {
    let root = read("src/typechecker/tests/resolver_metadata/metadata_restoration.rs");
    let aggregates =
        read("src/typechecker/tests/resolver_metadata/metadata_restoration/aggregates.rs");
    let behavior_refs =
        read("src/typechecker/tests/resolver_metadata/metadata_restoration/behavior_refs.rs");
    let callables =
        read("src/typechecker/tests/resolver_metadata/metadata_restoration/callables.rs");

    assert!(
        root.lines().count() < 80,
        "metadata_restoration.rs should only route focused restoration tests"
    );
    for module in ["mod aggregates;", "mod behavior_refs;", "mod callables;"] {
        assert!(
            root.contains(module),
            "metadata_restoration.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn resolver_struct_fields_from_metadata_restores_field_names_and_defaults"),
        "aggregate restoration tests should live in aggregates.rs"
    );
    assert!(
        aggregates.contains("fn resolver_enum_variants_from_metadata_uses_owner_scoped_payloads"),
        "aggregates.rs should cover enum and struct metadata restoration"
    );
    assert!(
        behavior_refs
            .contains("fn behavior_impl_refs_from_metadata_restores_type_and_behavior_keys"),
        "behavior_refs.rs should cover behavior ref metadata restoration"
    );
    assert!(
        callables.contains("fn resolver_params_from_metadata_preserves_ast_param_shape"),
        "callables.rs should cover callable metadata restoration"
    );
}

#[test]
fn resolver_metadata_requirement_tests_stay_split_by_responsibility() {
    let root = read("src/typechecker/tests/resolver_metadata/metadata_requirements.rs");
    let aggregates =
        read("src/typechecker/tests/resolver_metadata/metadata_requirements/aggregates.rs");
    let callables =
        read("src/typechecker/tests/resolver_metadata/metadata_requirements/callables.rs");
    let lookup = read("src/typechecker/tests/resolver_metadata/metadata_requirements/lookup.rs");

    assert!(
        root.lines().count() < 80,
        "metadata_requirements.rs should only route focused metadata requirement tests"
    );
    for module in ["mod aggregates;", "mod callables;", "mod lookup;"] {
        assert!(
            root.contains(module),
            "metadata_requirements.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn resolver_struct_field_metadata_requires_field_types"),
        "aggregate metadata requirement tests should live in aggregates.rs"
    );
    assert!(
        aggregates.contains("fn resolver_enum_variant_name_metadata_requires_variant_names"),
        "aggregates.rs should cover aggregate metadata requirements"
    );
    assert!(
        callables.contains("fn resolver_callable_signature_metadata_requires_complete_signature"),
        "callables.rs should cover callable and behavior method metadata requirements"
    );
    assert!(
        lookup.contains("fn resolver_behavior_ref_owner_prefers_exact_then_unique_fallbacks"),
        "lookup.rs should cover resolver metadata lookup helpers"
    );
}
