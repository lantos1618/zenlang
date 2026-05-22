use super::*;

#[test]
fn typechecker_resolve_docs_truth_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/typechecker_resolve.rs");
    let generic_validation =
        read("tests/docs_truth/repo_hygiene/typechecker_resolve/generic_validation.rs");
    let generic_walker =
        read("tests/docs_truth/repo_hygiene/typechecker_resolve/generic_walker.rs");
    let type_resolution =
        read("tests/docs_truth/repo_hygiene/typechecker_resolve/type_resolution.rs");

    assert!(
        root.lines().count() < 60,
        "typechecker_resolve.rs should only route focused typechecker resolve guard modules"
    );
    for module in [
        "mod generic_validation;",
        "mod generic_walker;",
        "mod type_resolution;",
    ] {
        assert!(
            root.contains(module),
            "typechecker_resolve.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn typechecker_binary_op_checking_lives_in_focused_helper"),
        "type resolution guards should live in type_resolution.rs"
    );
    assert!(
        type_resolution.contains("fn typechecker_type_resolution_uses_named_and_generic_helpers"),
        "type_resolution.rs should cover named and generic type resolution helpers"
    );
    assert!(
        generic_walker.contains("fn generic_type_reference_walker_bounds_live_in_focused_helper"),
        "generic_walker.rs should cover generic type-reference walker decomposition"
    );
    assert!(
        generic_validation.contains("fn generic_type_validation_ast_tasks_live_in_focused_helper"),
        "generic_validation.rs should cover AST type-reference validation guards"
    );
    assert!(
        generic_validation
            .contains("fn resolver_type_reference_collected_metadata_lives_in_focused_helper"),
        "generic_validation.rs should cover resolver-collected metadata guards"
    );
}
