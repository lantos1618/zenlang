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
fn resolver_behavior_ref_queue_helpers_live_in_focused_helper() {
    let support = read("src/typechecker/behavior_impl_support.rs");
    let resolver_refs = read("src/typechecker/behavior_impl_support/resolver_refs.rs");

    for helper in [
        "resolver_impl_ref_for",
        "resolver_behavior_ref_for",
        "behavior_ref_parts",
        "pop_resolver_behavior_ref",
        "peek_resolver_behavior_ref",
        "resolver_behavior_ref_queue_index",
        "named_queue_index",
        "named_queue_index_preserving_future_front",
        "resolver_behavior_impl_ref_parts",
    ] {
        assert!(
            !support.contains(&format!("fn {helper}")),
            "behavior_impl_support.rs should not own resolver behavior-ref queue helper: {helper}"
        );
        assert!(
            resolver_refs.contains(&format!("fn {helper}")),
            "resolver behavior-ref queue helper should live in focused helper: {helper}"
        );
    }

    assert!(
        support.lines().count() < 130,
        "behavior_impl_support.rs should stay focused on overlap and generic-target diagnostics"
    );
    assert!(
        support.contains("mod resolver_refs;"),
        "behavior impl support should load the focused resolver behavior-ref helper"
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

#[test]
fn behavior_association_inheritance_lives_in_focused_helper() {
    let root = read("src/typechecker/behavior_associations.rs");
    let focused = read("src/typechecker/behavior_associations/inheritance.rs");

    for helper in [
        "validate_behavior_extends_cycles",
        "behavior_extends_has_cycle",
        "validate_behavior_method_coherence",
        "collect_behavior_method_coherence_errors",
        "type_implements_behavior",
        "behavior_inherits_from",
        "behavior_inherits_from_inner",
        "behavior_extends_parent_matches",
        "behavior_ref_inherits_from_inner",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "behavior_associations.rs should not own inheritance/coherence helper: {helper}"
        );
        assert!(
            focused.contains(&format!("fn {helper}")),
            "behavior association inheritance/coherence helper should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("mod inheritance;"),
        "behavior associations should load the focused inheritance/coherence helper"
    );
}

#[test]
fn behavior_impl_signature_collection_lives_in_focused_helper() {
    let root = read("src/typechecker/mod.rs");
    let focused = read("src/typechecker/behavior_impl_signature_collection.rs");

    for helper in [
        "collect_impl_method_signature",
        "collect_resolver_backed_impl_method_template",
        "collect_resolver_behavior_impl_method_signatures",
        "collect_behavior_default_method_signatures",
        "should_skip_behavior_default_synthesis",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "typechecker root should not own behavior impl signature helper: {helper}"
        );
        assert!(
            focused.contains(&format!("fn {helper}")),
            "behavior impl signature collection should live in focused helper: {helper}"
        );
    }

    assert!(
        root.lines().count() < 190,
        "typechecker root should stay focused on module wiring and shared imports"
    );
    assert!(
        root.contains("mod behavior_impl_signature_collection;"),
        "typechecker root should include focused behavior impl signature collection"
    );
}

#[test]
fn behavior_impl_method_validation_lives_in_focused_helper() {
    let root = read("src/typechecker/behavior_impl_validation.rs");
    let methods = read("src/typechecker/behavior_impl_validation/methods.rs");

    for helper in [
        "fn check_behavior_impl_extra_methods(",
        "fn check_behavior_impl_required_methods(",
    ] {
        assert!(
            !root.contains(helper),
            "behavior_impl_validation.rs should not own method validation helper `{helper}`"
        );
        assert!(
            methods.contains(helper),
            "behavior impl method validation helper should live in focused helper: {helper}"
        );
    }

    assert!(
        root.lines().count() < 190,
        "behavior_impl_validation.rs should stay focused on impl validation orchestration"
    );
    assert!(
        root.contains("mod methods;"),
        "behavior impl validation should load the focused method validation helper"
    );
}
