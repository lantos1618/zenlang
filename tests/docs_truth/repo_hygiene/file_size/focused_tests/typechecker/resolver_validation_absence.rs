use super::*;

#[test]
fn resolver_validation_absence_tests_stay_split_by_metadata_owner() {
    let root = read("src/typechecker/tests/resolver_validation/absence.rs");
    let behavior_associations =
        read("src/typechecker/tests/resolver_validation/absence/behavior_associations.rs");
    let behavior_declarations =
        read("src/typechecker/tests/resolver_validation/absence/behavior_declarations.rs");

    assert!(
        root.lines().count() < 80,
        "absence.rs should only route focused absence-validation test modules"
    );
    for module in ["mod behavior_associations;", "mod behavior_declarations;"] {
        assert!(
            root.contains(module),
            "absence.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "behavior_association_absence_validation_builds_entries",
        "behavior_declaration_absence_validation_builds_entries",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "absence.rs should not own concrete test body: {test_name}"
        );
    }
    assert!(
        behavior_associations.contains("fn behavior_association_absence_validation_builds_entries")
            && behavior_associations
                .contains("fn behavior_association_absence_validation_uses_value_resolver_codes"),
        "behavior_associations.rs should cover association absence entries and resolver codes"
    );
    assert!(
        behavior_declarations.contains("fn behavior_declaration_absence_validation_builds_entries")
            && behavior_declarations
                .contains("fn behavior_declaration_absence_validation_uses_value_resolver_codes"),
        "behavior_declarations.rs should cover declaration absence entries and resolver codes"
    );
}
