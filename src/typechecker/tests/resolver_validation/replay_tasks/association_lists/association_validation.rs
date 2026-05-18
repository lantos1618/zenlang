use super::*;

#[test]
fn behavior_extends_replay_task_helper_pushes_parent_validation() {
    let program = parse_program(
        r#"
Json<T>: behavior {
}
Pretty<T>: behavior {
}

Pretty.extends(Json<T>)
"#,
    );
    let mut tasks = Vec::new();

    let handled =
        TypeChecker::push_behavior_extends_replay_task(&program.declarations[2], &mut tasks);

    assert!(handled);
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].behavior, "Pretty");
    assert_eq!(tasks[0].parent, "Json");
    assert_eq!(
        tasks[0].parent_type_args,
        &[AstType::Named("T".to_string())]
    );
}

#[test]
fn behavior_requires_replay_task_helper_pushes_requires_validation() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.requires(Json<StaticString>)
"#,
    );
    let mut tasks = Vec::new();

    let handled =
        TypeChecker::push_behavior_requires_replay_task(&program.declarations[2], &mut tasks);

    assert!(handled);
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].type_name, "Point");
    assert_eq!(tasks[0].behavior, "Json");
    assert_eq!(tasks[0].behavior_type_args, &[AstType::Str]);
}

#[test]
fn behavior_association_validation_tasks_collect_extends_impls_and_requires_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Pretty<T>: behavior {
    pretty: (Self) T
}

Pretty.extends(Json<T>)

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );

    let tasks = TypeChecker::collect_behavior_association_validation_tasks(&program.declarations);

    assert_eq!(tasks.extends.len(), 1);
    assert_eq!(tasks.extends[0].behavior, "Pretty");
    assert_eq!(tasks.extends[0].parent, "Json");
    assert_eq!(
        tasks.extends[0].parent_type_args,
        &[AstType::Named("T".to_string())]
    );
    assert_eq!(tasks.impls.len(), 1);
    assert_eq!(tasks.impls[0].ast_type_name, "Point");
    assert_eq!(tasks.impls[0].behavior, "Json");
    assert_eq!(tasks.impls[0].behavior_type_args, &[AstType::Str]);
    assert_eq!(tasks.requires.len(), 1);
    assert_eq!(tasks.requires[0].type_name, "Point");
    assert_eq!(tasks.requires[0].behavior, "Json");
    assert_eq!(tasks.requires[0].behavior_type_args, &[AstType::Str]);
}

#[test]
fn behavior_association_validation_helper_replays_impls_and_requires() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );
    let mut checker = TypeChecker::new();
    checker.collect_declarations(&program.declarations);
    let tasks = TypeChecker::collect_behavior_association_validation_tasks(&program.declarations);

    checker.validate_behavior_association_tasks(&tasks, None);

    assert!(
        checker.diagnostics().is_empty(),
        "valid impl+requires replay should not emit diagnostics: {:?}",
        checker.diagnostics()
    );
}
