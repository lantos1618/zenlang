use super::*;

#[test]
fn resolver_type_declaration_metadata_tasks_collect_only_type_work() {
    let program = parse_program(
        r#"
Point: { x: i32 = 1 }
Option<T>: Some(T), None

main = () i32 { 1 }
"#,
    );

    let tasks =
        TypeChecker::collect_resolver_type_declaration_metadata_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    assert!(matches!(
        tasks[0],
        ResolverTypeDeclarationMetadataTask::Struct { name: "Point", .. }
    ));
    assert!(matches!(
        tasks[1],
        ResolverTypeDeclarationMetadataTask::Enum { name: "Option", .. }
    ));
}

#[test]
fn resolver_type_replay_task_helper_pushes_metadata_and_type_refs_together() {
    let program = parse_program(
        r#"
Point: { x: i32 = 1 }
"#,
    );
    let mut type_tasks = Vec::new();
    let mut type_reference_tasks = Vec::new();

    let handled = TypeChecker::push_resolver_type_replay_tasks(
        &program.declarations[0],
        &mut type_tasks,
        &mut type_reference_tasks,
    );

    assert!(handled);
    assert!(matches!(
        type_tasks.as_slice(),
        [ResolverTypeDeclarationMetadataTask::Struct { name: "Point", .. }]
    ));
    assert!(matches!(
        type_reference_tasks.as_slice(),
        [ResolverTypeReferenceValidationTask::Struct { name: "Point", .. }]
    ));
}
