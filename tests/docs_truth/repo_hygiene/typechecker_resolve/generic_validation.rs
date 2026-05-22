use super::*;

#[test]
fn resolver_type_reference_collected_metadata_lives_in_focused_helper() {
    let root = read("src/typechecker/generic_type_validation/resolver_type_references.rs");
    let collected = read(
        "src/typechecker/generic_type_validation/resolver_type_references/collected_metadata.rs",
    );

    assert!(
        root.lines().count() < 180,
        "resolver_type_references.rs should stay focused on resolver task dispatch"
    );
    assert!(
        root.contains("mod collected_metadata;"),
        "resolver type-reference validation should include focused collected metadata helper"
    );
    for helper in [
        "collected_value_type_param_scope",
        "collected_type_type_param_scope",
        "collected_behavior_type_param_scope",
        "validate_collected_struct_type_references",
        "validate_collected_enum_type_references",
        "validate_collected_behavior_type_references",
        "validate_collected_value_type_references",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "resolver type-reference dispatch should not own collected metadata helper: {helper}"
        );
        assert!(
            collected.contains(&format!("pub(super) fn {helper}")),
            "collected metadata helper should own resolver metadata validation: {helper}"
        );
    }
}

#[test]
fn generic_type_validation_ast_tasks_live_in_focused_helper() {
    let root = read("src/typechecker/generic_type_validation.rs");
    let ast_tasks = read("src/typechecker/generic_type_validation/ast_type_references.rs");
    let validation =
        read("src/typechecker/generic_type_validation/ast_type_references/validation.rs");

    assert!(
        root.lines().count() < 120,
        "generic_type_validation.rs should stay focused on module wiring and resolver-name helpers"
    );
    assert!(
        root.contains("mod ast_type_references;"),
        "generic type validation should include focused AST type-reference task helper"
    );
    assert!(
        ast_tasks.lines().count() < 90,
        "AST type-reference task collection should not own task validation"
    );
    assert!(
        ast_tasks.contains("mod validation;"),
        "AST type-reference task collection should include focused validation helper"
    );
    for helper in [
        "collect_ast_type_reference_validation_tasks",
        "push_ast_type_reference_validation_task",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "generic type validation root should not own AST task helper: {helper}"
        );
        assert!(
            ast_tasks.contains(&format!("fn {helper}")),
            "AST type-reference task collection should own: {helper}"
        );
    }
    for helper in [
        "validate_ast_type_reference_tasks",
        "validate_ast_callable_type_references",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "generic type validation root should not own AST task helper: {helper}"
        );
        assert!(
            !ast_tasks.contains(&format!("fn {helper}")),
            "AST type-reference task collection should not own validation helper: {helper}"
        );
        assert!(
            validation.contains(&format!("fn {helper}")),
            "AST type-reference validation helper should own: {helper}"
        );
    }
}
