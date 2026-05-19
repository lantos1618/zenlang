use super::*;

#[test]
fn behavior_impl_generic_behavior_without_type_args_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior impl without type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 0")),
        "expected generic behavior impl arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_nongeneric_behavior_type_args_are_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json<i32>) {
    encode = (value: Point) StaticString { "point" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("non-generic behavior impl with type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic behavior `Json` does not accept type arguments")),
        "expected non-generic behavior impl type-argument diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("generic behavior `Json` expects 0")),
        "non-generic behavior impl should not use generic arity wording, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_with_type_args_passes_requires() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior impl should satisfy matching generic requires");
}

#[test]
fn behavior_impl_generic_behavior_type_arg_bound_failure_is_error() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { value }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior type argument bound should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json<Point>` required by `T`")),
        "expected generic behavior type argument bound diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Json<Point>) {
    encode = (value: Point) Point { value }
}

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { value }
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior type argument bound should pass when satisfied");
}

#[test]
fn behavior_impl_generic_behavior_substitutes_method_signature() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) i32 { 1 }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior impl return mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("method `encode` for behavior `Json_StaticString` expects return `StaticString`, found `i32`")),
        "expected substituted behavior method return diagnostic, got {errors:?}"
    );
}
