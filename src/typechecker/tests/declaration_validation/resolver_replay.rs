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

#[test]
fn resolver_behavior_impl_replay_task_helper_pushes_metadata_and_type_refs_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (self: Point) str { "point" }
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

#[test]
fn resolver_behavior_impl_block_declaration_tasks_collect_only_behavior_impls() {
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

    let tasks =
        TypeChecker::collect_resolver_behavior_impl_block_declaration_tasks(&program.declarations);

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].ast_type_name, "Point");
    assert_eq!(tasks[0].behavior, "Json");
    assert_eq!(tasks[0].methods.len(), 1);
}

#[test]
fn resolver_declaration_semantic_bundle_replays_validation_passes() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }

Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (self: Point) str { "point" }
}

Point.requires(Json)

main = (value: Point) i32 { 1 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    let mut stale_declarations = program.declarations.clone();
    if let Declaration::Requires { behavior, .. } = &mut stale_declarations[3] {
        *behavior = "MissingBehavior".to_string();
    }
    if let Declaration::Function { params, .. } = &mut stale_declarations[4] {
        params[0].ty = AstType::Named("MissingType".to_string());
    }
    let mut checker = TypeChecker::new();
    checker.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&stale_declarations)
    });
    checker.collect_resolver_declaration_metadata(&symbols, &tasks);

    checker.validate_resolver_declaration_semantics_from_tasks(&tasks, Some(&symbols));

    assert!(
        checker
            .diagnostics()
            .iter()
            .all(|d| !d.message.contains("MissingBehavior")),
        "resolver task bundle should not validate stale AST behavior refs, got {:?}",
        checker.diagnostics()
    );
    assert!(
        checker
            .diagnostics()
            .iter()
            .all(|d| !d.message.contains("MissingType")),
        "resolver task bundle should not validate stale AST type refs, got {:?}",
        checker.diagnostics()
    );
    assert!(
        checker.diagnostics().iter().any(|d| d
            .message
            .contains("field `x` default expects `i32`, found `bool`")),
        "expected field default diagnostics, got {:?}",
        checker.diagnostics()
    );
}

#[test]
fn resolver_declaration_collection_bundle_replays_metadata_semantics_and_refresh() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }

Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (self: Point) str { "point" }
}

Point.requires(Json)

main = (value: Point) i32 { 1 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    let mut stale_declarations = program.declarations.clone();
    if let Declaration::Requires { behavior, .. } = &mut stale_declarations[3] {
        *behavior = "MissingBehavior".to_string();
    }
    if let Declaration::Function { params, .. } = &mut stale_declarations[4] {
        params[0].ty = AstType::Named("MissingType".to_string());
    }
    let mut checker = TypeChecker::new();
    checker.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&stale_declarations)
    });

    checker.collect_resolver_declarations_from_tasks(&tasks, &symbols);

    assert!(
        checker
            .behavior_impls
            .contains(&("Point".to_string(), "Json".to_string())),
        "resolver collection bundle should refresh validated behavior impls"
    );
    assert!(
        checker
            .diagnostics()
            .iter()
            .all(|d| !d.message.contains("MissingBehavior")),
        "resolver collection bundle should not validate stale AST behavior refs, got {:?}",
        checker.diagnostics()
    );
    assert!(
        checker
            .diagnostics()
            .iter()
            .all(|d| !d.message.contains("MissingType")),
        "resolver collection bundle should not validate stale AST type refs, got {:?}",
        checker.diagnostics()
    );
    assert!(
        checker.diagnostics().iter().any(|d| d
            .message
            .contains("field `x` default expects `i32`, found `bool`")),
        "resolver collection bundle should validate resolver struct defaults, got {:?}",
        checker.diagnostics()
    );
}

#[test]
fn declaration_collection_replay_bundle_collects_ast_and_resolver_tasks_together() {
    let program = parse_program(
        r#"
{ io } = std

Point: { x: i32 }

Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (self: Point) str { "point" }
}

Point.requires(Json)

main = () i32 { 1 }
"#,
    );

    let tasks = TypeChecker::collect_declaration_collection_replay_tasks(&program.declarations);

    assert_eq!(tasks.ast.imports.len(), 1);
    assert_eq!(tasks.ast.types.len(), 1);
    assert_eq!(tasks.ast.behaviors.len(), 1);
    assert_eq!(tasks.ast.callable.len(), 1);
    assert_eq!(tasks.ast.impl_blocks.len(), 1);
    assert_eq!(tasks.resolver.types.len(), 1);
    assert_eq!(tasks.resolver.behaviors.len(), 1);
    assert_eq!(tasks.resolver.callable.len(), 1);
    assert_eq!(tasks.resolver.behavior_associations.impls.len(), 1);
    assert_eq!(tasks.resolver.behavior_associations.requires.len(), 1);
    assert_eq!(tasks.resolver.type_references.len(), 4);
}
