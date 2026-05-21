use super::*;

mod replay_execution;

#[test]
fn declaration_collection_replay_bundle_collects_ast_and_resolver_tasks_together() {
    let program = parse_program(
        r#"
{ io } = std

Point: { x: i32 }

Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (self: Point) StaticString { "point" }
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
    assert_eq!(
        tasks.resolver_semantics.behavior_associations.impls.len(),
        1
    );
    assert_eq!(
        tasks
            .resolver_semantics
            .behavior_associations
            .requires
            .len(),
        1
    );
    assert_eq!(tasks.resolver_semantics.struct_defaults.len(), 1);
    assert_eq!(tasks.resolver_semantics.type_references.len(), 4);
}

#[test]
fn resolver_declaration_semantic_tasks_collect_only_semantic_work() {
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

    let tasks =
        TypeChecker::collect_resolver_declaration_semantic_validation_tasks(&program.declarations);

    assert_eq!(tasks.behavior_associations.impls.len(), 1);
    assert_eq!(tasks.behavior_associations.requires.len(), 1);
    assert_eq!(tasks.struct_defaults.len(), 1);
    assert_eq!(tasks.struct_defaults[0].name, "Point");
    assert_eq!(tasks.type_references.len(), 4);
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
