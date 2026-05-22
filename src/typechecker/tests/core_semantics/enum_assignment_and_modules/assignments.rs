use super::*;

#[test]
fn assignment_to_immutable_binding_is_error() {
    let program = parse_program(
        r#"
main = () void {
    x = 1
    x = 2
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("immutable assignment should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("cannot assign to immutable variable `x`")),
        "expected immutable assignment diagnostic, got {errors:?}"
    );
}

#[test]
fn assignment_to_mutable_closure_parameter_is_allowed() {
    let program = parse_program(
        r#"
main = () void {
    mapper = (mut input: i32) i32 {
        input = input + 1
        input
    }
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("mutable closure parameter assignment should pass");
}

#[test]
fn assignment_type_mismatch_is_error() {
    let program = parse_program(
        r#"
main = () void {
    x ::= 1
    x = "bad"
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("assignment type mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("assignment to `x` expects `i32`, found `StaticString`")),
        "expected assignment type diagnostic, got {errors:?}"
    );
}
