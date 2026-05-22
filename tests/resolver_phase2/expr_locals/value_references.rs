use super::*;

#[test]
fn resolver_rejects_unknown_unqualified_function_calls() {
    let program = parse_program(
        r#"
known = () i32 { 1 }
main = () i32 { missing() }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown function call should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown value symbol 'missing'")),
        "expected missing function diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_unknown_local_identifier_references() {
    let program = parse_program(
        r#"
main = () i32 {
    missing_local
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown local identifier should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown value symbol 'missing_local'")),
        "expected missing local diagnostic, got {err:?}"
    );
}
