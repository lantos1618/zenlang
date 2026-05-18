use super::*;

#[test]
fn behavior_requires_generic_behavior_type_arg_arity_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.requires(Json<i32, StaticString>)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior requires arity mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 2")),
        "expected generic behavior requires arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_requires_nongeneric_behavior_type_args_are_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) StaticString
}

Point.requires(Json<i32>)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("non-generic behavior requires with type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic behavior `Json` does not accept type arguments")),
        "expected non-generic behavior requires type-argument diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("generic behavior `Json` expects 0")),
        "non-generic behavior requires should not use generic arity wording, got {errors:?}"
    );
}

#[test]
fn behavior_requires_passes_when_impl_exists() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.implements(Json) {
    to_json = (value: Point) StaticString { "point" }
}

Point.requires(Json)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("requires should pass when behavior impl exists");
}

#[test]
fn behavior_requires_rejects_missing_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.requires(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("requires should fail without behavior impl");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement required behavior `Json`")),
        "expected requires missing impl diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_requires_generic_behavior_without_type_args_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) StaticString
}

Point.requires(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior requires without type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 0")),
        "expected generic behavior requires arity diagnostic, got {errors:?}"
    );
}
