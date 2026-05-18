use super::*;

#[test]
fn range_expression_is_rejected_until_range_type_exists() {
    let program = parse_program(
        r#"
main = () i32 {
    1..3
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("range expressions should be gated until range typing exists");

    assert!(
        err.iter()
            .any(|d| d.message.contains("range expressions are not implemented")),
        "expected range diagnostic, got {err:?}"
    );
}

#[test]
fn result_raise_is_rejected_until_propagation_lowering_exists() {
    let program = parse_program(
        r#"
Result<T, E>:
    Ok(T),
    Err(E)

main = () i32 {
    value = Result<i32, str>.Ok(1)
    value.raise()
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("raise propagation should stay gated until lowering exists");

    assert!(
        err.iter()
            .any(|d| d.message.contains("`.raise()` is gated")),
        "expected raise gate diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .all(|d| !d.message.contains("has no method `raise`")),
        "raise gate should not be reported as an ordinary missing method, got {err:?}"
    );
}
