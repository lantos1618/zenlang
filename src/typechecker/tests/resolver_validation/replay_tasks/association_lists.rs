use super::*;

mod association_validation;

#[test]
fn resolver_behavior_association_list_tasks_collect_type_and_parent_edges_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    let tasks = TypeChecker::collect_resolver_behavior_association_list_tasks(&program, &symbols);

    assert_eq!(tasks.type_associations.len(), 1);
    let type_task = &tasks.type_associations[0];
    assert_eq!(type_task.name, "Point");
    assert_eq!(type_task.impl_edges[0].display, "Json<StaticString>");
    assert_eq!(type_task.required_edges[0].display, "Json<StaticString>");

    assert_eq!(tasks.behavior_parents.len(), 2);
    let pretty_task = tasks
        .behavior_parents
        .iter()
        .find(|task| task.name == "PrettyJson")
        .expect("PrettyJson parent task");
    assert_eq!(pretty_task.parent_edges[0].display, "Json<StaticString>");
    let json_task = tasks
        .behavior_parents
        .iter()
        .find(|task| task.name == "Json")
        .expect("Json empty parent task");
    assert!(json_task.parent_edges.is_empty());
}

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
fn resolver_behavior_association_list_tasks_select_from_declaration_bundle() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Pretty<T>: behavior {
    pretty: (Self) StaticString
}

Pretty.extends(Json<T>)

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    let declaration_tasks =
        TypeChecker::collect_resolver_validation_replay_declaration_tasks(&program, &symbols);
    let association_tasks =
        TypeChecker::collect_resolver_behavior_association_list_tasks_from_declaration_tasks(
            &declaration_tasks,
        );

    assert_eq!(association_tasks.type_associations.len(), 1);
    assert_eq!(association_tasks.type_associations[0].name, "Point");
    assert_eq!(
        association_tasks.type_associations[0].impl_edges[0].display,
        "Json<StaticString>"
    );
    assert_eq!(
        association_tasks.type_associations[0].required_edges[0].display,
        "Json<StaticString>"
    );
    assert_eq!(association_tasks.behavior_parents.len(), 2);
    let pretty_task = association_tasks
        .behavior_parents
        .iter()
        .find(|task| task.name == "Pretty")
        .expect("Pretty parent association task");
    assert_eq!(pretty_task.parent_edges[0].display, "Json<T>");
}

#[test]
fn resolver_type_reference_validation_tasks_collect_only_type_reference_work() {
    let program = parse_program(
        r#"
Point: { x: i32 }

main = (input: Point) Point {
    input
}
"#,
    );

    let tasks =
        TypeChecker::collect_resolver_type_reference_validation_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    assert!(matches!(
        tasks[0],
        ResolverTypeReferenceValidationTask::Struct { name: "Point", .. }
    ));
    assert!(matches!(
        tasks[1],
        ResolverTypeReferenceValidationTask::Function { name: "main", .. }
    ));
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

#[test]
fn resolver_declaration_metadata_tasks_collect_top_level_type_reference_tasks() {
    let program = parse_program(
        r#"
value := 1
"#,
    );

    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);

    assert!(
        tasks.type_references.iter().any(|task| matches!(
            task,
            ResolverTypeReferenceValidationTask::TopLevelExpr { .. }
        )),
        "top-level expression type references should stay in the shared resolver task collector"
    );
}
