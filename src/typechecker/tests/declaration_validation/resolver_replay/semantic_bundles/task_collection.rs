use super::*;

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
