use super::*;

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
