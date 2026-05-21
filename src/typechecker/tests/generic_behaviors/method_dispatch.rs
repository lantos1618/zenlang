use super::*;

#[test]
fn local_behavior_method_call_does_not_use_enclosing_return_type_to_pick_candidate() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(Json<i32>) {
    encode = (value: Point) i32 { value.x }
}

main = () i32 {
    point = Point { x: 1 }
    encoded = point.encode()
    0
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("ambiguous behavior method call should fail");

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("ambiguous behavior method `encode` for type `Point`")),
        "expected ambiguous behavior method diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("has no method `encode`")),
        "ambiguous behavior method call should not degrade to unknown method, got {errors:?}"
    );
}

#[test]
fn local_annotation_disambiguates_behavior_method_call() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(Json<i32>) {
    encode = (value: Point) i32 { value.x }
}

main = () i32 {
    point = Point { x: 1 }
    encoded: i32 = point.encode()
    encoded
}
"#,
    );

    let typed = TypeChecker::new()
        .check_program(&program)
        .expect("explicit local annotation should disambiguate behavior method call");
    let main = typed
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function");
    let encoded = main
        .body
        .statements
        .iter()
        .find_map(|statement| match &statement.kind {
            TypedStatementKind::VarDecl { name, value, .. } if name == "encoded" => Some(value),
            _ => None,
        })
        .expect("encoded local");

    assert!(
        matches!(
            &encoded.kind,
            TypedExprKind::FunctionCall { function, .. }
                if function == "Point.encode__Json_i32"
        ),
        "expected local annotation to select Json<i32> behavior method, got {encoded:?}"
    );
    assert_eq!(encoded.ty, Type::I32);
}

#[test]
fn assignment_target_disambiguates_behavior_method_call() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(Json<i32>) {
    encode = (value: Point) i32 { value.x }
}

main = () i32 {
    point = Point { x: 1 }
    encoded ::= 0
    encoded = point.encode()
    encoded
}
"#,
    );

    let typed = TypeChecker::new()
        .check_program(&program)
        .expect("assignment target type should disambiguate behavior method call");
    let main = typed
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function");
    let assignment = main
        .body
        .statements
        .iter()
        .find_map(|statement| match &statement.kind {
            TypedStatementKind::Expression(TypedExpression {
                kind: TypedExprKind::Assign { value, .. },
                ..
            }) => Some(value.as_ref()),
            _ => None,
        })
        .expect("encoded assignment");

    assert!(
        matches!(
            &assignment.kind,
            TypedExprKind::FunctionCall { function, .. }
                if function == "Point.encode__Json_i32"
        ),
        "expected assignment target to select Json<i32> behavior method, got {assignment:?}"
    );
    assert_eq!(assignment.ty, Type::I32);
}
