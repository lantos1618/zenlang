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
