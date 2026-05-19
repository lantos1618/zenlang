use super::*;

#[test]
fn typechecker_ast_behavior_collection_lives_in_focused_helper() {
    let ast_collection = read("src/typechecker/declaration_collection_ast.rs");
    let behavior_collection = read("src/typechecker/declaration_collection_ast_behaviors.rs");

    for helper in [
        "collect_behavior_declaration_tasks",
        "push_behavior_declaration_task",
        "collect_behavior_declarations_from_tasks",
        "collect_ast_behavior_declaration_signature",
        "collect_resolver_backed_behavior_declaration_stub",
        "validate_ast_precollection_tasks",
        "collect_ast_precollection_validation_tasks",
        "push_behavior_extends_replay_task",
    ] {
        assert!(
            !ast_collection.contains(&format!("fn {helper}")),
            "AST declaration collection should not own behavior helper: {helper}"
        );
        assert!(
            behavior_collection.contains(&format!("fn {helper}")),
            "AST behavior collection should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/mod.rs");
    assert!(
        root.contains("mod declaration_collection_ast_behaviors;"),
        "typechecker root should load focused AST behavior collection"
    );
}
