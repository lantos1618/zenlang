use super::*;

#[test]
fn resolver_behavior_ref_validation_descriptor_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let behavior_refs = read("src/typechecker/resolver_validation_support/behavior_refs.rs");
    let validation = read("src/typechecker/resolver_validation_support/behavior_ref_validation.rs");

    for helper in [
        "BehaviorRefValidation",
        "BehaviorRefRole",
        "BehaviorRefCheck",
    ] {
        assert!(
            !behavior_refs.contains(&format!("struct {helper}"))
                && !behavior_refs.contains(&format!("enum {helper}")),
            "behavior_refs.rs should not own validation descriptor helper: {helper}"
        );
        assert!(
            validation.contains(&format!("struct {helper}"))
                || validation.contains(&format!("enum {helper}")),
            "behavior-ref validation descriptor helper should live in focused helper: {helper}"
        );
    }

    assert!(
        behavior_refs.lines().count() < 180,
        "behavior_refs.rs should stay focused on expected behavior-edge metadata"
    );
    assert!(
        root.contains("include!(\"resolver_validation_support/behavior_ref_validation.rs\");"),
        "resolver validation support should include focused behavior-ref validation helper"
    );
}

#[test]
fn resolver_behavior_absence_descriptors_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation_support.rs");
    let mixed = read("src/typechecker/resolver_validation_support/absence_symbol_descriptors.rs");
    let behavior =
        read("src/typechecker/resolver_validation_support/behavior_absence_descriptors.rs");

    for helper in [
        "BehaviorAssociationAbsenceValidation",
        "BehaviorDeclarationAbsenceValidation",
    ] {
        assert!(
            !mixed.contains(&format!("struct {helper}")),
            "absence_symbol_descriptors.rs should not own behavior absence descriptor: {helper}"
        );
        assert!(
            behavior.contains(&format!("struct {helper}")),
            "behavior absence descriptor should live in focused helper: {helper}"
        );
    }

    assert!(
        mixed.lines().count() < 140,
        "absence_symbol_descriptors.rs should stay focused on non-behavior descriptors"
    );
    assert!(
        root.contains("include!(\"resolver_validation_support/behavior_absence_descriptors.rs\");"),
        "resolver validation support should include focused behavior absence descriptors"
    );
}
