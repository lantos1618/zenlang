use super::*;

#[test]
fn resolver_behavior_declaration_metadata_tasks_collect_only_behavior_work() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

main = () i32 { 1 }
"#,
    );

    let tasks =
        TypeChecker::collect_resolver_behavior_declaration_metadata_tasks(&program.declarations);

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].name, "Json");
}

#[test]
fn resolver_behavior_replay_task_helper_pushes_metadata_and_type_refs_together() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );
    let mut behavior_tasks = Vec::new();
    let mut type_reference_tasks = Vec::new();

    let handled = TypeChecker::push_resolver_behavior_replay_tasks(
        &program.declarations[0],
        &mut behavior_tasks,
        &mut type_reference_tasks,
    );

    assert!(handled);
    assert_eq!(behavior_tasks.len(), 1);
    assert_eq!(behavior_tasks[0].name, "Json");
    assert!(matches!(
        type_reference_tasks.as_slice(),
        [ResolverTypeReferenceValidationTask::Behavior { name: "Json", .. }]
    ));
}
