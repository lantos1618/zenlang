use super::*;

#[test]
fn invalid_field_access_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

main = () void {
    p = Point { x: 1 }
    y = p.y
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("invalid field access should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("type `Point` has no field `y`")),
        "expected invalid field diagnostic, got {errors:?}"
    );
}

#[test]
fn non_void_function_without_return_is_error() {
    let program = parse_program(
        r#"
missing = () i32 {
    x = 1
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-void fallthrough should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("function `missing` must return `i32` on all non-error paths")),
        "expected missing return diagnostic, got {errors:?}"
    );
}
