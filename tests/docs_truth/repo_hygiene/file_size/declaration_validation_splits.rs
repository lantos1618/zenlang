use super::super::*;

#[test]
fn resolver_semantic_bundle_task_tests_live_in_focused_helper() {
    let root =
        read("src/typechecker/tests/declaration_validation/resolver_replay/semantic_bundles.rs");
    let task_collection = read(
        "src/typechecker/tests/declaration_validation/resolver_replay/semantic_bundles/task_collection.rs",
    );

    for test_name in [
        "declaration_collection_replay_bundle_collects_ast_and_resolver_tasks_together",
        "resolver_declaration_semantic_tasks_collect_only_semantic_work",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "semantic_bundles.rs should not own task collection assertion: {test_name}"
        );
        assert!(
            task_collection.contains(&format!("fn {test_name}")),
            "resolver semantic bundle task assertions should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 190,
        "semantic_bundles.rs should stay focused on resolver-backed replay execution"
    );
    assert!(
        root.contains("mod task_collection;"),
        "semantic_bundles.rs should include the focused task_collection module"
    );
}

#[test]
fn resolver_replay_behavior_task_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/declaration_validation/resolver_replay.rs");
    let behavior_tasks =
        read("src/typechecker/tests/declaration_validation/resolver_replay/behavior_tasks.rs");

    for test_name in [
        "resolver_behavior_declaration_metadata_tasks_collect_only_behavior_work",
        "resolver_behavior_replay_task_helper_pushes_metadata_and_type_refs_together",
        "resolver_behavior_impl_replay_task_helper_pushes_metadata_and_type_refs_together",
        "resolver_behavior_impl_block_declaration_tasks_collect_only_behavior_impls",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_replay.rs should not own behavior replay task test: {test_name}"
        );
        assert!(
            behavior_tasks.contains(&format!("fn {test_name}")),
            "behavior replay task tests should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 150,
        "resolver_replay.rs should stay focused on type and callable replay task helpers"
    );
    assert!(
        root.contains("mod behavior_tasks;"),
        "resolver_replay.rs should include the focused behavior_tasks module"
    );
}
