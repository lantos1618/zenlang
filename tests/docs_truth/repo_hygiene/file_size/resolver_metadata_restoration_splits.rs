use super::super::*;

#[test]
fn resolver_metadata_restoration_behavior_ref_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/resolver_metadata/metadata_restoration.rs");
    let behavior_refs =
        read("src/typechecker/tests/resolver_metadata/metadata_restoration/behavior_refs.rs");

    for test_name in [
        "behavior_parent_refs_from_metadata_restores_keys_and_type_args",
        "behavior_impl_refs_from_metadata_restores_type_and_behavior_keys",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "metadata_restoration.rs should not own behavior-ref restoration test: {test_name}"
        );
        assert!(
            behavior_refs.contains(&format!("fn {test_name}")),
            "behavior-ref restoration tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 210,
        "metadata_restoration.rs should stay focused on callable, enum, struct, and behavior method restoration"
    );
    assert!(
        root.contains("mod behavior_refs;"),
        "metadata_restoration.rs should include the focused behavior_refs module"
    );
}
