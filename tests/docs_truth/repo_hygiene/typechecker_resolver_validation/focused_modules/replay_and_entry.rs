use super::*;

#[test]
fn typechecker_resolver_entry_association_helpers_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let entry = read("src/typechecker/resolver_validation/entry_symbols.rs");
    let associations = read("src/typechecker/resolver_validation/entry_associations.rs");

    for helper in [
        "validate_resolver_impl_block_entry",
        "validate_resolver_requires_entry",
        "validate_resolver_behavior_extends_entry",
    ] {
        assert!(
            !entry.contains(&format!("fn {helper}")),
            "resolver entry traversal should not own behavior-association helper: {helper}"
        );
        assert!(
            entry.contains(&format!("self.{helper}(")),
            "resolver entry traversal should delegate behavior-association work through {helper}"
        );
        assert!(
            associations.contains(&format!("fn {helper}")),
            "resolver behavior-association entry helper should live in focused helper: {helper}"
        );
    }

    assert!(
        entry.lines().count() < 220,
        "resolver entry traversal should stay focused on declaration dispatch"
    );
    assert!(
        root.contains("include!(\"resolver_validation/entry_associations.rs\");"),
        "resolver validation should include focused entry behavior-association helpers"
    );
}

#[test]
fn typechecker_resolver_replay_association_tasks_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let replay = read("src/typechecker/resolver_validation/replay_tasks.rs");
    let associations = read("src/typechecker/resolver_validation/replay_task_associations.rs");
    let declarations = read("src/typechecker/resolver_validation/replay_task_declarations.rs");

    for helper in [
        "collect_resolver_behavior_association_list_tasks_from_declaration_tasks",
        "push_resolver_type_behavior_association_list_task",
        "push_resolver_behavior_parent_list_task",
    ] {
        assert!(
            !replay.contains(&format!("fn {helper}")),
            "resolver replay task root should not own behavior-association replay helper: {helper}"
        );
        assert!(
            associations.contains(&format!("fn {helper}")),
            "behavior-association replay helper should live in focused helper: {helper}"
        );
    }

    assert!(
        !replay.contains("fn collect_resolver_validation_replay_declaration_tasks"),
        "resolver replay task root should delegate declaration replay collection"
    );
    assert!(
        declarations.contains("fn collect_resolver_validation_replay_declaration_tasks"),
        "declaration replay collection should live in focused helper"
    );
    assert!(
        declarations.contains("push_expected_resolver_callable_symbol"),
        "declaration replay helper should own callable declaration replay collection"
    );
    assert!(
        declarations.contains("push_expected_behavior_impl_edge"),
        "declaration replay helper should own behavior impl edge collection"
    );
    assert!(
        replay.lines().count() < 80,
        "resolver replay task root should only coordinate replay task collection"
    );
    assert!(
        root.contains("include!(\"resolver_validation/replay_task_associations.rs\");"),
        "resolver validation should include focused behavior-association replay helpers"
    );
    assert!(
        root.contains("include!(\"resolver_validation/replay_task_declarations.rs\");"),
        "resolver validation should include focused declaration replay helper"
    );
}
