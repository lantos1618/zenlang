use super::*;

#[test]
fn typechecker_struct_default_validation_lives_in_focused_helper() {
    let semantic = read("src/typechecker/semantic_validation.rs");
    let defaults = read("src/typechecker/semantic_validation_struct_defaults.rs");

    for helper in [
        "validate_struct_field_defaults",
        "collect_ast_struct_field_default_validation_tasks",
        "push_ast_struct_field_default_validation_task",
        "validate_ast_struct_field_default_tasks",
        "validate_resolver_struct_field_defaults",
        "validate_resolver_struct_field_default_task_list",
        "validate_ast_struct_field_defaults",
        "validate_struct_field_default",
    ] {
        assert!(
            !semantic.contains(&format!("fn {helper}")),
            "semantic validation dispatch should not own struct default helper: {helper}"
        );
        assert!(
            defaults.contains(&format!("fn {helper}")),
            "struct default semantic validation should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/mod.rs");
    assert!(
        root.contains("mod semantic_validation;"),
        "typechecker should still load semantic validation root"
    );
}

#[test]
fn typechecker_self_type_validation_tasks_live_in_focused_helper() {
    let root = read("src/typechecker/self_type_validation.rs");
    let tasks = read("src/typechecker/self_type_validation/tasks.rs");

    for helper in [
        "collect_self_type_context_validation_tasks",
        "push_self_type_context_validation_task",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "self_type_validation.rs should not own self-type task collection helper: {helper}"
        );
        assert!(
            tasks.contains(&format!("fn {helper}")),
            "self-type task collection should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("mod tasks;"),
        "self_type_validation.rs should load focused task collection helper"
    );
    assert!(
        root.lines().count() < 230,
        "self_type_validation.rs should stay focused on self-type validation"
    );
}
