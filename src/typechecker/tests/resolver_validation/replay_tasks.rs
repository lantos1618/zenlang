use super::*;

#[test]
fn resolver_behavior_association_list_tasks_collect_type_and_parent_edges_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    let tasks = TypeChecker::collect_resolver_behavior_association_list_tasks(&program, &symbols);

    assert_eq!(tasks.type_associations.len(), 1);
    let type_task = &tasks.type_associations[0];
    assert_eq!(type_task.name, "Point");
    assert_eq!(type_task.impl_edges[0].display, "Json<str>");
    assert_eq!(type_task.required_edges[0].display, "Json<str>");

    assert_eq!(tasks.behavior_parents.len(), 2);
    let pretty_task = tasks
        .behavior_parents
        .iter()
        .find(|task| task.name == "PrettyJson")
        .expect("PrettyJson parent task");
    assert_eq!(pretty_task.parent_edges[0].display, "Json<str>");
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

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
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
        "Json<str>"
    );
    assert_eq!(
        tasks
            .expected_associations
            .required
            .owned_edges_for("Point")[0]
            .display,
        "Json<str>"
    );
    assert_eq!(tasks.behavior_declarations.len(), 1);
    assert_eq!(tasks.behavior_declarations[0].name, "Json");
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

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)

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
    assert_eq!(type_task.impl_edges[0].display, "Json<str>");
    assert_eq!(type_task.required_edges[0].display, "Json<str>");
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

Point.requires(Json<str>)
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

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
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

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
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

#[test]
fn impl_block_declaration_tasks_collect_behavior_and_plain_impls() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) str
}

Point.impl = {
    x_value = (value: Point) i32 { value.x }
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );

    let tasks = TypeChecker::collect_impl_block_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].type_name, "Point");
    assert_eq!(tasks[0].behavior, None);
    assert_eq!(tasks[1].type_name, "Point");
    assert_eq!(tasks[1].behavior, Some("Json"));
    assert_eq!(tasks[1].methods.len(), 1);
}

#[test]
fn callable_declaration_tasks_collect_functions_and_methods() {
    let program = parse_program(
        r#"
Point: { x: i32 }

make = () Point { Point { x: 1 } }

Point.get = (self: Point) i32 { self.x }
"#,
    );

    let tasks = TypeChecker::collect_callable_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    match &tasks[0] {
        CallableDeclarationTask::Function { name, .. } => assert_eq!(*name, "make"),
        _ => panic!("expected function task"),
    }
    match &tasks[1] {
        CallableDeclarationTask::Method {
            type_name,
            method_name,
            ..
        } => {
            assert_eq!(*type_name, "Point");
            assert_eq!(*method_name, "get");
        }
        _ => panic!("expected method task"),
    }
}

#[test]
fn ast_type_declaration_tasks_collect_structs_and_enums() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Option<T>: Some(T), None
"#,
    );

    let tasks = TypeChecker::collect_ast_type_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    match &tasks[0] {
        AstTypeDeclarationTask::Struct { name, fields, .. } => {
            assert_eq!(*name, "Point");
            assert_eq!(fields.len(), 1);
        }
        _ => panic!("expected struct task"),
    }
    match &tasks[1] {
        AstTypeDeclarationTask::Enum {
            name,
            type_params,
            variants,
        } => {
            assert_eq!(*name, "Option");
            assert_eq!(type_params.len(), 1);
            assert_eq!(variants.len(), 2);
        }
        _ => panic!("expected enum task"),
    }
}

#[test]
fn behavior_declaration_tasks_collect_behavior_signatures() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );

    let tasks = TypeChecker::collect_behavior_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].name, "Json");
    assert_eq!(tasks[0].type_params.len(), 1);
    assert_eq!(tasks[0].methods.len(), 1);
    assert_eq!(tasks[0].methods[0].name, "encode");
}
