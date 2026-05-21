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

#[test]
fn behavior_association_inheritance_lives_in_focused_helper() {
    let root = read("src/typechecker/behavior_associations.rs");
    let inheritance = read("src/typechecker/behavior_associations/inheritance.rs");
    let implementation_queries =
        read("src/typechecker/behavior_associations/implementation_queries.rs");

    for helper in [
        "validate_behavior_extends_cycles",
        "behavior_extends_has_cycle",
        "validate_behavior_method_coherence",
        "collect_behavior_method_coherence_errors",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "behavior_associations.rs should not own inheritance/coherence helper: {helper}"
        );
        assert!(
            inheritance.contains(&format!("fn {helper}")),
            "behavior association inheritance/coherence helper should live in focused helper: {helper}"
        );
    }

    for helper in [
        "type_implements_behavior",
        "behavior_inherits_from",
        "behavior_inherits_from_inner",
        "behavior_extends_parent_matches",
        "behavior_ref_inherits_from_inner",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "behavior_associations.rs should not own implementation query helper: {helper}"
        );
        assert!(
            !inheritance.contains(&format!("fn {helper}")),
            "inheritance.rs should not own implementation query helper: {helper}"
        );
        assert!(
            implementation_queries.contains(&format!("fn {helper}")),
            "behavior implementation query helper should live in focused helper: {helper}"
        );
    }

    assert!(
        inheritance.lines().count() < 140,
        "behavior association inheritance helper should stay focused on inheritance validation"
    );
    assert!(
        root.contains("mod inheritance;"),
        "behavior associations should load the focused inheritance/coherence helper"
    );
    assert!(
        root.contains("mod implementation_queries;"),
        "behavior associations should load the focused implementation query helper"
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
    let root = read("src/typechecker/mod.rs");
    let validation = read("src/typechecker/behavior_impl_validation.rs");
    let methods = read("src/typechecker/behavior_impl_method_validation.rs");

    assert!(
        validation.lines().count() < 135,
        "behavior impl validation should stay focused on impl setup and duplicate/overlap checks"
    );

    for helper in [
        "validate_behavior_impl_methods",
        "validate_behavior_impl_declared_methods",
        "validate_behavior_impl_required_method",
        "behavior_impl_actual_method_signature",
    ] {
        assert!(
            !validation.contains(&format!("fn {helper}")),
            "behavior impl validation should not own method validation helper: {helper}"
        );
        assert!(
            methods.contains(&format!("fn {helper}")),
            "behavior impl method validation should live in focused helper: {helper}"
        );
    }

    for diagnostic in [
        "method `{}` is not declared by behavior `{}`",
        "type `{}` implementation of `{}` is missing required method `{}`",
        "method `{}` for behavior `{}` expects {} parameters, found {}",
        "parameter {} for method `{}` in behavior `{}` expects `{}`, found `{}`",
        "method `{}` for behavior `{}` expects return `{}`, found `{}`",
    ] {
        assert!(
            methods.contains(diagnostic),
            "behavior impl method validation should own method diagnostic: {diagnostic}"
        );
    }

    assert!(
        root.contains("mod behavior_impl_method_validation;"),
        "typechecker root should include focused behavior impl method validation"
    );
}
