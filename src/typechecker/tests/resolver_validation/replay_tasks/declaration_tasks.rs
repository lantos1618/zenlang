use super::*;

#[test]
fn impl_block_declaration_tasks_collect_behavior_and_plain_impls() {
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
