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
            "fn resolver_behavior_impl_replay_task_helper_pushes_metadata_and_type_refs_together",
        ),
        "behavior_impls.rs should cover behavior impl replay task pairing"
    );
    assert!(
        callables.contains(
            "fn resolver_callable_replay_task_helper_pushes_metadata_and_type_refs_together",
        ),
        "callables.rs should cover callable replay task pairing"
    );
}
