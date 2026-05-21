use super::*;

#[test]
fn generic_behavior_bound_with_type_args_accepts_matching_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<Point>) {
    encode = (value: Point) Point { value }
}

identity<T: Json<T>> = (value: T) T {
    value
}

main = () i32 {
    p = Point { x: 1 }
    same = identity(p)
    same.x
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior bound type argument should substitute at call site");
}

#[test]
fn generic_behavior_bound_with_type_args_rejects_mismatched_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

identity<T: Json<T>> = (value: T) T {
    value
}

main = () i32 {
    p = Point { x: 1 }
    same = identity(p)
    same.x
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior bound should require matching behavior type args");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json<Point>` required by `T`")),
        "expected generic behavior bound type argument diagnostic, got {errors:?}"
    );
}
