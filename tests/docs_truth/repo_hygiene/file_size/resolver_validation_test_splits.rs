use super::super::*;

#[test]
fn behavior_ref_role_diagnostic_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/resolver_validation/behavior_refs.rs");
    let role_validation =
        read("src/typechecker/tests/resolver_validation/behavior_refs/role_validation.rs");

    for test_name in [
        "behavior_ref_validation_maps_role_and_check_diagnostics",
        "behavior_ref_validation_separates_role_labels_from_check_codes",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "behavior_refs.rs should not own role diagnostic test: {test_name}"
        );
        assert!(
            role_validation.contains(&format!("fn {test_name}")),
            "behavior-ref role diagnostic test should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 180,
        "behavior_refs.rs should stay focused on actual metadata selection tests"
    );
    assert!(
        root.contains("mod role_validation;"),
        "behavior_refs.rs should include the focused role_validation module"
    );
}
