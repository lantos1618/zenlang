use super::super::*;

#[test]
fn resolver_validation_docs_truth_stays_split_across_focused_modules() {
    let root = read("tests/docs_truth/repo_hygiene/typechecker_resolver_validation.rs");

    assert!(
        root.lines().count() < 260,
        "typechecker resolver-validation docs-truth guards should stay split across focused modules"
    );
}

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
fn declaration_validation_precollection_tasks_live_in_focused_helper() {
    let tasks = read("src/typechecker/tests/declaration_validation/tasks.rs");
    let precollection = read("src/typechecker/tests/declaration_validation/precollection_tasks.rs");
    let module = read("src/typechecker/tests/declaration_validation.rs");

    assert!(
        tasks.lines().count() < 240,
        "tasks.rs should stay focused on declaration semantic validation tasks"
    );
    assert!(
        !tasks.contains("self_type_context_validation_tasks_collect_declarations"),
        "precollection task tests should live in precollection_tasks.rs"
    );
    assert!(
        precollection.contains("self_type_context_validation_tasks_collect_declarations"),
        "precollection_tasks.rs should cover self type context task collection"
    );
    assert!(
        precollection.contains("ast_declaration_collection_bundle_replays_collection_passes"),
        "precollection_tasks.rs should cover declaration collection task replay"
    );
    assert!(
        module.contains("mod precollection_tasks;"),
        "declaration_validation.rs should include the focused precollection_tasks module"
    );
}

#[test]
fn struct_literal_default_tests_live_in_focused_helper() {
    let struct_literals = read("src/typechecker/tests/core_semantics/struct_literals.rs");
    let defaults = read("src/typechecker/tests/core_semantics/struct_literal_defaults.rs");
    let module = read("src/typechecker/tests/core_semantics.rs");

    assert!(
        struct_literals.lines().count() < 180,
        "struct_literals.rs should stay focused on struct literal error cases"
    );
    assert!(
        !struct_literals.contains("struct_literal_uses_default_for_omitted_field"),
        "struct literal default tests should live in struct_literal_defaults.rs"
    );
    assert!(
        defaults.contains("struct_literal_uses_default_for_omitted_field"),
        "struct_literal_defaults.rs should cover defaulted field omission"
    );
    assert!(
        defaults.contains("generic_struct_literal_uses_substituted_default_for_omitted_field"),
        "struct_literal_defaults.rs should cover generic default substitution"
    );
    assert!(
        module.contains("mod struct_literal_defaults;"),
        "core_semantics.rs should include the focused struct_literal_defaults module"
    );
}

#[test]
fn generic_behavior_impl_type_arg_tests_live_in_focused_helper() {
    let impls = read("src/typechecker/tests/generic_behaviors/impls_and_requires.rs");
    let type_args = read("src/typechecker/tests/generic_behaviors/impl_type_args.rs");
    let module = read("src/typechecker/tests/generic_behaviors.rs");

    assert!(
        impls.lines().count() < 180,
        "impls_and_requires.rs should stay focused on basic impl/require behavior tests"
    );
    assert!(
        !impls.contains("behavior_impl_generic_behavior_without_type_args_is_error"),
        "generic behavior impl type-argument tests should live in impl_type_args.rs"
    );
    assert!(
        type_args.contains("behavior_impl_generic_behavior_without_type_args_is_error"),
        "impl_type_args.rs should cover missing generic behavior type arguments"
    );
    assert!(
        type_args.contains("behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied"),
        "impl_type_args.rs should cover satisfied generic behavior type-argument bounds"
    );
    assert!(
        module.contains("mod impl_type_args;"),
        "generic_behaviors.rs should include the focused impl_type_args module"
    );
}

#[test]
fn generic_behavior_bound_tests_stay_split_by_responsibility() {
    let root = read("src/typechecker/tests/generic_behaviors/generic_bounds.rs");
    let type_args = read("src/typechecker/tests/generic_behaviors/generic_bounds/type_args.rs");
    let declarations =
        read("src/typechecker/tests/generic_behaviors/generic_bounds/declarations.rs");
    let call_site = read("src/typechecker/tests/generic_behaviors/generic_bounds/call_site.rs");
    let collection =
        read("src/typechecker/tests/generic_behaviors/generic_bounds/collection_metadata.rs");

    for module in [
        "mod call_site;",
        "mod collection_metadata;",
        "mod declarations;",
        "mod type_args;",
    ] {
        assert!(
            root.contains(module),
            "generic_bounds.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("generic_behavior_bound_with_type_args_accepts_matching_impl"),
        "generic behavior type-argument bound tests should live in type_args.rs"
    );
    assert!(
        type_args.contains("generic_behavior_bound_with_type_args_rejects_mismatched_impl"),
        "type_args.rs should cover generic behavior bound type-argument mismatches"
    );
    assert!(
        declarations.contains("behavior_generic_bound_unknown_behavior_reports_once"),
        "declarations.rs should cover generic bound declaration diagnostics"
    );
    assert!(
        call_site.contains("generic_behavior_bound_accepts_inherited_behavior_impl"),
        "call_site.rs should cover inherited impl satisfaction"
    );
    assert!(
        collection.contains("func_info_non_generic_has_empty_type_params"),
        "collection_metadata.rs should cover non-generic function metadata"
    );
}
