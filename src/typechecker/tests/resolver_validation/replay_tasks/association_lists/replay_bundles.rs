use super::*;

#[test]
fn resolver_expected_symbol_sets_collect_declarations_and_locals_together() {
    let program = parse_program(
        r#"
{ io } = std

Point: { x: i32 = 1 }

main = (input: i32) i32 {
    value := input
    value
}
"#,
    );

    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let expected =
        TypeChecker::collect_resolver_validation_replay_tasks(&program, &symbols).expected_symbols;

    assert!(expected.validate_imports);
    assert!(expected
        .declarations
        .contains(&(Namespace::Module, "std".to_string())));
    assert!(expected
        .declarations
        .contains(&(Namespace::Import, "io".to_string())));
    assert!(expected
        .declarations
        .contains(&(Namespace::Type, "Point".to_string())));
    assert!(expected
        .declarations
        .contains(&(Namespace::Value, "main".to_string())));
    assert!(expected.locals.contains(&("input".to_string(), 2)));
    assert!(expected.locals.contains(&("value".to_string(), 3)));
}

#[test]
fn resolver_validation_replay_declaration_tasks_collect_sources_and_edges() {
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
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    let tasks =
        TypeChecker::collect_resolver_validation_replay_declaration_tasks(&program, &symbols);

    assert!(tasks
        .expected_symbols
        .declarations
        .contains(&(Namespace::Type, "Point".to_string())));
    assert_eq!(tasks.type_declarations.len(), 1);
    assert_eq!(tasks.type_declarations[0].name, "Point");
    assert_eq!(
        tasks.expected_associations.impls.owned_edges_for("Point")[0].display,
        "Json<StaticString>"
    );
    assert_eq!(
        tasks
            .expected_associations
            .required
            .owned_edges_for("Point")[0]
            .display,
        "Json<StaticString>"
    );
    assert_eq!(tasks.behavior_declarations.len(), 1);
    assert_eq!(tasks.behavior_declarations[0].name, "Json");
}

#[test]
fn resolver_validation_replay_tasks_collect_symbols_and_behavior_associations_together() {
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

main = (input: i32) i32 {
    input
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    let tasks = TypeChecker::collect_resolver_validation_replay_tasks(&program, &symbols);

    assert!(tasks
        .expected_symbols
        .declarations
        .contains(&(Namespace::Type, "Point".to_string())));
    assert!(tasks
        .expected_symbols
        .declarations
        .contains(&(Namespace::Behavior, "Json".to_string())));
    assert!(tasks
        .expected_symbols
        .declarations
        .contains(&(Namespace::Value, "main".to_string())));
    assert!(tasks
        .expected_symbols
        .locals
        .iter()
        .any(|(name, _)| name == "input"));

    let type_task = &tasks.behavior_associations.type_associations[0];
    assert_eq!(type_task.name, "Point");
    assert_eq!(type_task.impl_edges[0].display, "Json<StaticString>");
    assert_eq!(type_task.required_edges[0].display, "Json<StaticString>");
    assert_eq!(tasks.behavior_associations.behavior_parents.len(), 1);
}
