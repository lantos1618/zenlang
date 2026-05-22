use super::*;

#[test]
fn enum_variant_unknown_variant_is_error() {
    let program = parse_program(
        r#"
Status: Ok, Err

main = () void {
    value = Status.Pending
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown enum variant should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("enum `Status` has no variant `Pending`")),
        "expected unknown variant diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_variant_payload_type_mismatch_is_error() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

main = () void {
    value = Maybe.Some("bad")
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("enum payload type mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("payload for enum variant `Maybe.Some` expects `i32`, found `StaticString`")),
        "expected payload type diagnostic, got {errors:?}"
    );
}
