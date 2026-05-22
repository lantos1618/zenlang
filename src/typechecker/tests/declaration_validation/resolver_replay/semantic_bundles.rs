use super::*;

mod task_collection;

#[test]
fn resolver_declaration_semantic_bundle_replays_validation_passes() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }

Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (self: Point) StaticString { "point" }
}

Point.requires(Json)

main = (value: Point) i32 { 1 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let metadata_tasks =
        TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    let semantic_tasks =
        TypeChecker::collect_resolver_declaration_semantic_validation_tasks(&program.declarations);
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
    checker.collect_resolver_declaration_metadata(&symbols, &metadata_tasks);

    checker.validate_resolver_declaration_semantics_from_semantic_tasks(
        &semantic_tasks,
        Some(&symbols),
    );

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
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (self: Point) StaticString { "point" }
}

Point.requires(Json)

main = (value: Point) i32 { 1 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let tasks = TypeChecker::collect_declaration_collection_replay_tasks(&program.declarations);
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

    checker.collect_resolver_declarations_from_tasks(
        &tasks.resolver,
        &tasks.resolver_semantics,
        &symbols,
    );

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
fn resolver_declaration_semantic_bundle_replays_dedicated_semantic_tasks() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }

Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (self: Point) StaticString { "point" }
}

Point.requires(Json)

main = (value: Point) i32 { 1 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let mut checker = TypeChecker::new();
    checker.collect_declarations_with_symbols(&program.declarations, &symbols);
    checker.diagnostics.clear();
    let tasks =
        TypeChecker::collect_resolver_declaration_semantic_validation_tasks(&program.declarations);

    checker.validate_resolver_declaration_semantics_from_semantic_tasks(&tasks, Some(&symbols));

    assert!(
        checker.diagnostics().iter().any(|d| d
            .message
            .contains("field `x` default expects `i32`, found `bool`")),
        "resolver semantic task bundle should validate struct defaults, got {:?}",
        checker.diagnostics()
    );
}
