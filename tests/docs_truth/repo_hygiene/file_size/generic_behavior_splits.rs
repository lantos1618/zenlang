use super::super::*;

#[test]
fn generic_behavior_bound_type_arg_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/generic_behaviors/generic_bounds.rs");
    let type_args = read("src/typechecker/tests/generic_behaviors/generic_bounds/type_args.rs");
    let module = read("src/typechecker/tests/generic_behaviors.rs");

    for test_name in [
        "generic_behavior_bound_with_type_args_accepts_matching_impl",
        "generic_behavior_bound_with_type_args_rejects_mismatched_impl",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "generic_bounds.rs should not own behavior-bound type-argument test: {test_name}"
        );
        assert!(
            type_args.contains(&format!("fn {test_name}")),
            "generic behavior-bound type-argument tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 210,
        "generic_bounds.rs should stay focused on declaration and implementation bound semantics"
    );
    assert!(
        root.contains("mod type_args;"),
        "generic_bounds.rs should include the focused type_args module"
    );
    assert!(
        module.contains("mod generic_bounds;"),
        "generic_behaviors.rs should include generic bound tests"
    );
}
