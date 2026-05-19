use super::*;

#[test]
fn behavior_default_method_synthesis_lives_in_focused_helper() {
    let support = read("src/typechecker/behavior_impl_support.rs");
    let defaults = read("src/typechecker/behavior_impl_support/default_methods.rs");

    for helper in [
        "behavior_default_methods_for_impl",
        "seed_behavior_default_method_signature",
        "impl_methods_include_behavior_method",
        "behavior_methods_with_inherited_substituted",
        "behavior_parent_type_param_substitutions",
    ] {
        assert!(
            !support.contains(&format!("fn {helper}")),
            "behavior impl support should not own default-method synthesis helper: {helper}"
        );
        assert!(
            defaults.contains(&format!("fn {helper}")),
            "default behavior method synthesis should live in focused helper: {helper}"
        );
    }

    assert!(
        support.contains("mod default_methods;"),
        "behavior impl support should load the focused default-method helper"
    );
}

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
