use super::super::*;

#[test]
fn resolver_behavior_ref_validation_message_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/resolver_validation/behavior_refs.rs");
    let messages =
        read("src/typechecker/tests/resolver_validation/behavior_refs/validation_messages.rs");

    for test_name in [
        "behavior_ref_validation_maps_role_and_check_diagnostics",
        "behavior_ref_validation_separates_role_labels_from_check_codes",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "behavior_refs.rs should not own validation message test: {test_name}"
        );
        assert!(
            messages.contains(&format!("fn {test_name}")),
            "validation message test should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 180,
        "behavior_refs.rs should stay focused on behavior ref metadata selection"
    );
    assert!(
        root.contains("mod validation_messages;"),
        "behavior_refs.rs should include focused validation message tests"
    );
}
