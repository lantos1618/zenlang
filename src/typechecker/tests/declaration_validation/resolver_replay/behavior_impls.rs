use super::*;

#[test]
fn resolver_behavior_impl_replay_task_helper_pushes_metadata_and_type_refs_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (self: Point) StaticString { "point" }
}
"#,
    );
    let mut behavior_impl_tasks = Vec::new();
    let mut type_reference_tasks = Vec::new();

    let handled = TypeChecker::push_resolver_behavior_impl_replay_tasks(
        &program.declarations[2],
        &mut behavior_impl_tasks,
        &mut type_reference_tasks,
    );

    assert!(handled);
    assert_eq!(behavior_impl_tasks.len(), 1);
    assert_eq!(behavior_impl_tasks[0].ast_type_name, "Point");
    assert_eq!(behavior_impl_tasks[0].behavior, "Json");
    assert!(matches!(
        type_reference_tasks.as_slice(),
        [ResolverTypeReferenceValidationTask::ImplBlock {
            type_name: "Point",
            ..
        }]
    ));
}

#[test]
fn resolver_behavior_impl_block_declaration_tasks_collect_only_behavior_impls() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) StaticString
}

Point.impl = {
    x_value = (value: Point) i32 { value.x }
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}
"#,
    );

    let tasks =
        TypeChecker::collect_resolver_behavior_impl_block_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].ast_type_name, "Point");
    assert_eq!(tasks[0].behavior, "Json");
    assert_eq!(tasks[0].methods.len(), 1);
}
