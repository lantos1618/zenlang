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
