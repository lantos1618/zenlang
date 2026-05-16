use super::*;

#[test]
fn check_program_rejects_self_type_outside_method_or_behavior() {
    let program = parse_program(
        r#"
main = (value: Self) i32 { 0 }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("Self should require a method or behavior context");

    assert!(
        err.iter()
            .any(|d| d.message.contains("Self type is only valid")),
        "expected invalid Self type diagnostic, got {err:?}"
    );
}

#[test]
fn self_type_context_validation_tasks_collect_declarations() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Option<T>: Some(T), None

Json: behavior {
    encode: (Self) str
}

Pretty.extends(Json)

make = () Point { Point { x: 1 } }

Point.get = (self: Point) i32 { self.x }

Point.impl = {
    x_value = (self: Point) i32 { self.x }
}

Point.requires(Json)

result := 1
"#,
    );

    let tasks = TypeChecker::collect_self_type_context_validation_tasks(&program.declarations);

    assert_eq!(tasks.len(), 9);
    assert!(matches!(
        tasks[0],
        SelfTypeContextValidationTask::Struct { .. }
    ));
    assert!(matches!(
        tasks[1],
        SelfTypeContextValidationTask::Enum { .. }
    ));
    assert!(matches!(
        tasks[2],
        SelfTypeContextValidationTask::Behavior { .. }
    ));
    assert!(matches!(
        tasks[3],
        SelfTypeContextValidationTask::BehaviorExtends { .. }
    ));
    assert!(matches!(
        tasks[4],
        SelfTypeContextValidationTask::Function { .. }
    ));
    assert!(matches!(
        tasks[5],
        SelfTypeContextValidationTask::Method { .. }
    ));
    assert!(matches!(
        tasks[6],
        SelfTypeContextValidationTask::ImplBlock { .. }
    ));
    assert!(matches!(
        tasks[7],
        SelfTypeContextValidationTask::Requires { .. }
    ));
    assert!(matches!(
        tasks[8],
        SelfTypeContextValidationTask::TopLevelExpr { .. }
    ));
}

#[test]
fn ast_precollection_validation_tasks_collect_self_and_extends_work() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) str
}

Pretty.extends(Json)

Point.impl = {
    x_value = (self: Point) i32 { self.x }
}

result := 1
"#,
    );

    let tasks = TypeChecker::collect_ast_precollection_validation_tasks(&program.declarations);

    assert_eq!(tasks.self_type_contexts.len(), 5);
    assert_eq!(tasks.behavior_associations.extends.len(), 1);
    assert_eq!(tasks.behavior_associations.extends[0].behavior, "Pretty");
    assert_eq!(tasks.behavior_associations.extends[0].parent, "Json");
    assert!(tasks.behavior_associations.impls.is_empty());
    assert!(tasks.behavior_associations.requires.is_empty());
}

#[test]
fn ast_declaration_collection_tasks_include_precollection_validation_work() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) str
}

Pretty.extends(Json)

make = () Point { Point { x: 1 } }
"#,
    );

    let tasks = TypeChecker::collect_ast_declaration_collection_tasks(&program.declarations);

    assert_eq!(tasks.types.len(), 1);
    assert_eq!(tasks.behaviors.len(), 1);
    assert_eq!(tasks.callable.len(), 1);
    assert_eq!(tasks.precollection_validations.self_type_contexts.len(), 4);
    assert_eq!(
        tasks
            .precollection_validations
            .behavior_associations
            .extends
            .len(),
        1
    );
}

#[test]
fn check_program_rejects_unknown_type_references() {
    let program = parse_program(
        r#"
main = (value: Missing, items: Bag<i32>) i32 { 0 }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("unknown type reference should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown type diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Bag'")),
        "expected unknown generic type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_rejects_unknown_type_references_in_struct_field_defaults() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T = {
        same: Missing = 1
        same
    }
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("unknown struct field default type reference should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown field default type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_rejects_struct_field_default_type_mismatch() {
    let program = parse_program(
        r#"
Point: { x: i32 = "bad" }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("struct field default type mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("field `x` default expects `i32`, found `str`")),
        "expected field default type mismatch diagnostic, got {err:?}"
    );
}

#[test]
fn ast_struct_field_default_validation_tasks_collect_structs() {
    let program = parse_program(
        r#"
Point: { x: i32 = 1 }
Box<T>: { value: T }
"#,
    );

    let tasks =
        TypeChecker::collect_ast_struct_field_default_validation_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].type_params.len(), 0);
    assert_eq!(tasks[0].fields.len(), 1);
    assert_eq!(tasks[1].type_params.len(), 1);
    assert_eq!(tasks[1].fields.len(), 1);
}

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
fn ast_type_reference_validation_tasks_collect_declarations() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Option<T>: Some(T), None

Json<T>: behavior {
    encode: (Self) T
}

make = () Point { Point { x: 1 } }

Point.get = (self: Point) i32 { self.x }

Point.impl = {
    x_value = (self: Point) i32 { self.x }
}

result := make()
"#,
    );

    let tasks = TypeChecker::collect_ast_type_reference_validation_tasks(&program.declarations);

    assert_eq!(tasks.len(), 7);
    assert!(matches!(
        tasks[0],
        AstTypeReferenceValidationTask::Struct { .. }
    ));
    assert!(matches!(
        tasks[1],
        AstTypeReferenceValidationTask::Enum { .. }
    ));
    assert!(matches!(
        tasks[2],
        AstTypeReferenceValidationTask::Behavior { .. }
    ));
    assert!(matches!(
        tasks[3],
        AstTypeReferenceValidationTask::Function { .. }
    ));
    assert!(matches!(
        tasks[4],
        AstTypeReferenceValidationTask::Method { .. }
    ));
    assert!(matches!(
        tasks[5],
        AstTypeReferenceValidationTask::ImplBlock { .. }
    ));
    assert!(matches!(
        tasks[6],
        AstTypeReferenceValidationTask::TopLevelExpr { .. }
    ));
}

#[test]
fn ast_declaration_validation_tasks_collect_semantic_validation_work() {
    let program = parse_program(
        r#"
Point: { x: i32 = 1 }
Option<T>: Some(T), None

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (self: Point) str { "point" }
}

Point.requires(Json<str>)
JsonString.extends(Json<str>)

main = () i32 { 1 }
"#,
    );

    let tasks = TypeChecker::collect_ast_declaration_validation_tasks(&program.declarations);

    assert_eq!(tasks.behavior_associations.extends.len(), 1);
    assert_eq!(tasks.behavior_associations.impls.len(), 1);
    assert_eq!(tasks.behavior_associations.requires.len(), 1);
    assert_eq!(tasks.type_references.len(), 5);
    assert_eq!(tasks.struct_field_defaults.len(), 1);
}
