use super::*;

#[test]
fn generic_function_explicit_type_arg_arity_is_error() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T {
    value
}

main = () i32 {
    identity<i32, str>(1)
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("wrong generic type-argument arity should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic function `identity` expects 1 type arguments, found 2")),
        "expected generic arity diagnostic, got {errors:?}"
    );
}

#[test]
fn nongeneric_function_explicit_type_args_are_error() {
    let program = parse_program(
        r#"
id = (value: i32) i32 {
    value
}

main = () i32 {
    id<i32>(1)
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-generic function type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic function `id` does not accept type arguments")),
        "expected non-generic type-argument diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_inference_failure_is_error() {
    let program = parse_program(
        r#"
make_default<T> = () T {
    0
}

main = () i32 {
    make_default()
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("uninferred generic type argument should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("cannot infer type argument `T` for generic function `make_default`")),
        "expected generic inference diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_bound_references_unknown_behavior_is_error() {
    let program = parse_program(
        r#"
show<T: Display> = (value: T) T {
    value
}

main = () i32 {
    show(1)
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown generic behavior bounds should fail");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "generic bound `Display` on type parameter `T` references undefined behavior"
        )),
        "expected generic bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_bound_rejects_unspecialized_generic_behavior() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    to_json: (Self) str
}

encode<T: Json> = (value: T) str {
    "encoded"
}

main = () i32 {
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("generic behavior bound without type arguments should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("generic behavior `Json` expects 1 type arguments, found 0")
        }),
        "expected generic behavior bound arity diagnostic, got {errors:?}"
    );
}
