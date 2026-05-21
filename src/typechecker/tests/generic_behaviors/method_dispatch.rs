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
