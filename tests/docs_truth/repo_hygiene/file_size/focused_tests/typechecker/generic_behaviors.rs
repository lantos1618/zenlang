use super::*;

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

#[test]
fn generic_behavior_method_dispatch_tests_stay_split_by_dispatch_surface() {
    let root = read("src/typechecker/tests/generic_behaviors/method_dispatch.rs");
    let ambiguity = read("src/typechecker/tests/generic_behaviors/method_dispatch/ambiguity.rs");
    let context =
        read("src/typechecker/tests/generic_behaviors/method_dispatch/context_disambiguation.rs");

    assert!(
        root.lines().count() < 80,
        "method_dispatch.rs should only route focused behavior method dispatch tests"
    );
    for module in ["mod ambiguity;", "mod context_disambiguation;"] {
        assert!(
            root.contains(module),
            "method_dispatch.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "local_behavior_method_call_does_not_use_enclosing_return_type_to_pick_candidate",
        "local_annotation_disambiguates_behavior_method_call",
        "assignment_target_disambiguates_behavior_method_call",
        "function_argument_type_disambiguates_behavior_method_call",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "method_dispatch.rs should not own concrete test body: {test_name}"
        );
    }
    assert!(
        ambiguity.contains(
            "fn local_behavior_method_call_does_not_use_enclosing_return_type_to_pick_candidate",
        ),
        "ambiguity.rs should cover ambiguous behavior method dispatch"
    );
    assert!(
        context.contains("fn local_annotation_disambiguates_behavior_method_call"),
        "context_disambiguation.rs should cover local annotation dispatch context"
    );
    assert!(
        context.contains("fn assignment_target_disambiguates_behavior_method_call"),
        "context_disambiguation.rs should cover assignment target dispatch context"
    );
    assert!(
        context.contains("fn function_argument_type_disambiguates_behavior_method_call"),
        "context_disambiguation.rs should cover function argument dispatch context"
    );
}
