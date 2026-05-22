use super::*;

#[test]
fn resolver_behavior_parent_tests_stay_split_by_metadata_surface() {
    let root = read("src/typechecker/tests/resolver_behavior_parents.rs");
    let parent_metadata =
        read("src/typechecker/tests/resolver_behavior_parents/parent_metadata.rs");
    let extra_metadata = read("src/typechecker/tests/resolver_behavior_parents/extra_metadata.rs");

    assert!(
        root.lines().count() < 80,
        "resolver_behavior_parents.rs should only route focused parent metadata tests"
    );
    for module in ["mod extra_metadata;", "mod parent_metadata;"] {
        assert!(
            root.contains(module),
            "resolver_behavior_parents.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "check_program_with_symbols_validates_resolver_behavior_parent_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_parent_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_parent_refs",
        "check_program_with_symbols_accepts_resolver_behavior_parent_child_type_param_refs",
        "check_program_with_symbols_rejects_extra_resolver_behavior_parent_names",
        "check_program_with_symbols_rejects_extra_resolver_behavior_parent_refs",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_behavior_parents.rs should not own concrete test body: {test_name}"
        );
    }
    assert!(
        parent_metadata
            .contains("fn check_program_with_symbols_validates_resolver_behavior_parent_names"),
        "parent_metadata.rs should cover behavior parent name metadata"
    );
    assert!(
        parent_metadata.contains(
            "fn check_program_with_symbols_validates_resolver_generic_behavior_parent_refs",
        ),
        "parent_metadata.rs should cover generic behavior parent refs"
    );
    assert!(
        parent_metadata.contains(
            "fn check_program_with_symbols_accepts_resolver_behavior_parent_child_type_param_refs",
        ),
        "parent_metadata.rs should cover parent refs that use child type parameters"
    );
    assert!(
        extra_metadata
            .contains("fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_names"),
        "extra_metadata.rs should cover extra parent name metadata"
    );
    assert!(
        extra_metadata
            .contains("fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_refs"),
        "extra_metadata.rs should cover extra parent refs"
    );
}
