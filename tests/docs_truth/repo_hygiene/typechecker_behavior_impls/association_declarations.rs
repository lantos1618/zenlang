use super::*;

#[test]
fn behavior_association_declaration_tasks_live_in_focused_helper() {
    let root = read("src/typechecker/declaration_tasks.rs");
    let focused = read("src/typechecker/declaration_tasks_behavior_associations.rs");

    for helper in [
        "BehaviorAssociationValidationTasks",
        "BehaviorAssociationValidationTaskSource",
        "ResolverBehaviorImplBlockDeclarationTask",
        "ResolverBehaviorImplBlockTask",
        "ImplBlockDeclarationTask",
        "BehaviorRequiresValidationTask",
        "EffectiveBehaviorImplMethod",
        "BehaviorExtendsValidationTask",
    ] {
        assert!(
            !root.contains(&format!("struct {helper}"))
                && !root.contains(&format!("enum {helper}"))
                && !root.contains(&format!("trait {helper}")),
            "declaration_tasks.rs should not own behavior association task: {helper}"
        );
        assert!(
            focused.contains(&format!("struct {helper}"))
                || focused.contains(&format!("enum {helper}"))
                || focused.contains(&format!("trait {helper}")),
            "behavior association task should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("include!(\"declaration_tasks_behavior_associations.rs\");"),
        "declaration tasks should include focused behavior association tasks"
    );
}
