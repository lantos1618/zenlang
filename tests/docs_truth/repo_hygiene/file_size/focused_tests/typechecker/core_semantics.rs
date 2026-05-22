use super::*;

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
fn declaration_validation_resolver_replay_tests_stay_split_by_task_kind() {
    let root = read("src/typechecker/tests/declaration_validation/resolver_replay.rs");
    let behavior_impls =
        read("src/typechecker/tests/declaration_validation/resolver_replay/behavior_impls.rs");
    let behaviors =
        read("src/typechecker/tests/declaration_validation/resolver_replay/behaviors.rs");
    let callables =
        read("src/typechecker/tests/declaration_validation/resolver_replay/callables.rs");
    let type_declarations =
        read("src/typechecker/tests/declaration_validation/resolver_replay/type_declarations.rs");

    assert!(
        root.lines().count() < 80,
        "resolver_replay.rs should only route focused resolver replay task tests"
    );
    for module in [
        "mod behavior_impls;",
        "mod behaviors;",
        "mod callables;",
        "mod semantic_bundles;",
        "mod type_declarations;",
    ] {
        assert!(
            root.contains(module),
            "resolver_replay.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn resolver_type_declaration_metadata_tasks_collect_only_type_work"),
        "type replay task tests should live in type_declarations.rs"
    );
    assert!(
        type_declarations
            .contains("fn resolver_type_replay_task_helper_pushes_metadata_and_type_refs_together"),
        "type_declarations.rs should cover type replay task pairing"
    );
    assert!(
        behaviors.contains(
            "fn resolver_behavior_replay_task_helper_pushes_metadata_and_type_refs_together"
        ),
        "behaviors.rs should cover behavior replay task pairing"
    );
    assert!(
        behavior_impls.contains(
            "fn resolver_behavior_impl_replay_task_helper_pushes_metadata_and_type_refs_together"
        ),
        "behavior_impls.rs should cover behavior impl replay task pairing"
    );
    assert!(
        callables.contains(
            "fn resolver_callable_replay_task_helper_pushes_metadata_and_type_refs_together"
        ),
        "callables.rs should cover callable replay task pairing"
    );
}

#[test]
fn intrinsic_gate_tests_stay_split_by_effect_family() {
    let root = read("src/typechecker/tests/core_semantics/intrinsic_gates.rs");
    let async_scheduler =
        read("src/typechecker/tests/core_semantics/intrinsic_gates/async_scheduler.rs");
    let atomics = read("src/typechecker/tests/core_semantics/intrinsic_gates/atomics.rs");
    let raw_memory = read("src/typechecker/tests/core_semantics/intrinsic_gates/raw_memory.rs");
    let raw_pointers = read("src/typechecker/tests/core_semantics/intrinsic_gates/raw_pointers.rs");
    let syscalls = read("src/typechecker/tests/core_semantics/intrinsic_gates/syscalls.rs");

    assert!(
        root.lines().count() < 80,
        "intrinsic_gates.rs should only route focused intrinsic gate tests"
    );
    for module in [
        "mod async_scheduler;",
        "mod atomics;",
        "mod raw_memory;",
        "mod raw_pointers;",
        "mod syscalls;",
        "mod type_match;",
    ] {
        assert!(
            root.contains(module),
            "intrinsic_gates.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn async_scheduler_intrinsics_are_rejected_as_gated_not_unknown"),
        "async scheduler gate tests should live in async_scheduler.rs"
    );
    assert!(
        async_scheduler.contains("fn async_scheduler_intrinsics_are_rejected_as_gated_not_unknown"),
        "async_scheduler.rs should cover async scheduler gates"
    );
    assert!(
        raw_memory.contains("fn byte_memory_intrinsics_are_rejected_as_allocator_gates"),
        "raw_memory.rs should cover allocator-backed byte memory gates"
    );
    assert!(
        raw_pointers.contains("fn raw_pointer_intrinsics_are_rejected_as_ownership_gates"),
        "raw_pointers.rs should cover raw pointer ownership gates"
    );
    assert!(
        atomics.contains("fn atomic_intrinsics_are_rejected_as_effect_gates"),
        "atomics.rs should cover atomic effect gates"
    );
    assert!(
        syscalls.contains("fn syscall_intrinsics_are_rejected_as_host_effect_gates"),
        "syscalls.rs should cover syscall host-effect gates"
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
fn type_helper_tests_stay_split_by_semantic_surface() {
    let root = read("src/typechecker/tests/core_semantics/type_helpers.rs");
    let compatibility = read("src/typechecker/tests/core_semantics/type_helpers/compatibility.rs");
    let literal_coercion =
        read("src/typechecker/tests/core_semantics/type_helpers/literal_coercion.rs");
    let resolution = read("src/typechecker/tests/core_semantics/type_helpers/resolution.rs");
    let substitution = read("src/typechecker/tests/core_semantics/type_helpers/substitution.rs");

    assert!(
        root.lines().count() < 80,
        "type_helpers.rs should only route focused type-helper tests"
    );
    for module in [
        "mod compatibility;",
        "mod literal_coercion;",
        "mod resolution;",
        "mod substitution;",
    ] {
        assert!(
            root.contains(module),
            "type_helpers.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn types_compatible_basics"),
        "type compatibility tests should live in compatibility.rs"
    );
    assert!(
        compatibility.contains("fn static_string_literal_does_not_implicitly_allocate_string"),
        "compatibility.rs should cover StaticString/String allocation boundaries"
    );
    assert!(
        literal_coercion.contains("fn literal_coercion_in_var_decl"),
        "literal_coercion.rs should cover declaration literal coercion"
    );
    assert!(
        resolution.contains("fn infer_type_args_basic"),
        "resolution.rs should cover generic type-argument inference"
    );
    assert!(
        substitution
            .contains("fn substitute_type_preserves_function_type_arguments_in_nested_generics"),
        "substitution.rs should cover nested generic function-type substitution"
    );
}

#[test]
fn match_semantics_tests_stay_split_by_match_subject() {
    let root = read("src/typechecker/tests/core_semantics/match_semantics.rs");
    let bool_matches = read("src/typechecker/tests/core_semantics/match_semantics/bool_matches.rs");
    let enum_matches = read("src/typechecker/tests/core_semantics/match_semantics/enum_matches.rs");

    assert!(
        root.lines().count() < 80,
        "match_semantics.rs should only route focused match semantics tests"
    );
    for module in ["mod bool_matches;", "mod enum_matches;"] {
        assert!(
            root.contains(module),
            "match_semantics.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn enum_match_missing_variant_is_error"),
        "enum match tests should live in enum_matches.rs"
    );
    assert!(
        enum_matches.contains("fn enum_match_payload_shape_is_checked"),
        "enum_matches.rs should cover enum payload validation"
    );
    assert!(
        bool_matches.contains("fn bool_match_missing_arm_is_error_for_value_match"),
        "bool_matches.rs should cover bool exhaustiveness"
    );
    assert!(
        bool_matches.contains("fn match_arm_return_does_not_force_never_result_type"),
        "bool_matches.rs should cover match result typing"
    );
}
