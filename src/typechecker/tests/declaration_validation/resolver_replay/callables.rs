use super::*;

#[test]
fn resolver_callable_declaration_metadata_tasks_collect_callable_work() {
    let program = parse_program(
        r#"
Point: { x: i32 }

make = () Point { Point { x: 1 } }

Point.get = (self: Point) i32 { self.x }

Point.impl = {
    plus = (self: Point, other: Point) i32 { self.x + other.x }
}
"#,
    );

    let tasks =
        TypeChecker::collect_resolver_callable_declaration_metadata_tasks(&program.declarations);

    assert_eq!(tasks.len(), 3);
    assert!(matches!(
        tasks[0],
        ResolverCallableDeclarationMetadataTask::Function { name: "make", .. }
    ));
    assert!(matches!(
        tasks[1],
        ResolverCallableDeclarationMetadataTask::Method {
            type_name: "Point",
            method_name: "get",
            ..
        }
    ));
    assert!(matches!(
        tasks[2],
        ResolverCallableDeclarationMetadataTask::TypeImpl {
            type_name: "Point",
            ..
        }
    ));
}

#[test]
fn resolver_callable_replay_task_helper_pushes_metadata_and_type_refs_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

make = () Point { Point { x: 1 } }
"#,
    );
    let mut callable_tasks = Vec::new();
    let mut type_reference_tasks = Vec::new();

    let handled = TypeChecker::push_resolver_callable_replay_tasks(
        &program.declarations[1],
        &mut callable_tasks,
        &mut type_reference_tasks,
    );

    assert!(handled);
    assert!(matches!(
        callable_tasks.as_slice(),
        [ResolverCallableDeclarationMetadataTask::Function { name: "make", .. }]
    ));
    assert!(matches!(
        type_reference_tasks.as_slice(),
        [ResolverTypeReferenceValidationTask::Function { name: "make", .. }]
    ));
}
