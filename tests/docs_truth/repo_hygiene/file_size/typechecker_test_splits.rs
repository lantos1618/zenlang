use super::super::*;

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
fn resolver_metadata_validation_descriptor_tests_live_in_focused_modules() {
    let root = read("src/typechecker/tests/resolver_metadata/validation_descriptors.rs");
    let type_variants =
        read("src/typechecker/tests/resolver_metadata/validation_descriptors/type_variants.rs");

    for test_name in [
        "field_validation_formats_messages",
        "field_validation_uses_resolver_codes",
        "variant_payload_validation_formats_messages",
        "variant_payload_validation_uses_resolver_codes",
        "variant_owner_validation_formats_message",
        "variant_owner_validation_uses_resolver_code",
        "variant_name_validation_formats_message",
        "variant_name_validation_uses_resolver_code",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "validation_descriptors.rs should not own type/variant descriptor test: {test_name}"
        );
        assert!(
            type_variants.contains(&format!("fn {test_name}")),
            "type/variant descriptor tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 230,
        "validation_descriptors.rs should stay focused on shared/value descriptor tests"
    );
    assert!(
        root.contains("mod type_variants;"),
        "validation_descriptors.rs should include the focused type_variants module"
    );
}

#[test]
fn resolver_import_absence_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/resolver_import_metadata.rs");
    let absence = read("src/typechecker/tests/resolver_import_metadata/absent_metadata.rs");

    for test_name in [
        "check_program_with_symbols_validates_resolver_import_absent_declaration_metadata",
        "check_program_with_symbols_validates_resolver_import_absent_type_metadata",
        "check_program_with_symbols_validates_resolver_import_and_module_absent_mutability",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_import_metadata.rs should not own import absence test: {test_name}"
        );
        assert!(
            absence.contains(&format!("fn {test_name}")),
            "resolver import absence tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 170,
        "resolver_import_metadata.rs should stay focused on import source/visibility metadata"
    );
    assert!(
        root.contains("mod absent_metadata;"),
        "resolver_import_metadata.rs should include the focused absent_metadata module"
    );
}

#[test]
fn resolver_function_value_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/resolver_impl_values.rs");
    let function_values = read("src/typechecker/tests/resolver_impl_values/function_values.rs");

    for test_name in [
        "check_program_with_symbols_validates_resolver_function_arity",
        "check_program_with_symbols_validates_resolver_function_parameter_types",
        "check_program_with_symbols_validates_resolver_function_type_parameter_metadata",
        "check_program_with_symbols_validates_resolver_function_parameter_names",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_impl_values.rs should not own function value test: {test_name}"
        );
        assert!(
            function_values.contains(&format!("fn {test_name}")),
            "resolver function value tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 180,
        "resolver_impl_values.rs should stay focused on impl-method and enum-variant checks"
    );
    assert!(
        root.contains("mod function_values;"),
        "resolver_impl_values.rs should include the focused function_values module"
    );
}

#[test]
fn resolver_required_stale_diagnostic_tests_live_in_focused_helper() {
    let root =
        read("src/typechecker/tests/resolver_collection/behavior_impls/required_diagnostics.rs");
    let stale_requires = read(
        "src/typechecker/tests/resolver_collection/behavior_impls/required_diagnostics/stale_requires.rs",
    );

    for test_name in [
        "collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_required_metadata",
        "collect_declarations_with_symbols_does_not_validate_stale_requires_after_target_restore",
        "collect_declarations_with_symbols_uses_resolver_behavior_required_name_metadata",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "required_diagnostics.rs should not own stale requires diagnostic test: {test_name}"
        );
        assert!(
            stale_requires.contains(&format!("fn {test_name}")),
            "stale requires diagnostic tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 190,
        "required_diagnostics.rs should stay focused on resolver-restored requires diagnostics"
    );
    assert!(
        root.contains("mod stale_requires;"),
        "required_diagnostics.rs should include the focused stale_requires module"
    );
}
