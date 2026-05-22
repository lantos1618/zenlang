use super::*;

#[test]
fn resolver_rejects_unknown_enum_variant_expressions() {
    let program = parse_program(
        r#"
Status: Ok, Err

main = () i32 {
    value = Status.Pending
    0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown enum variant expression should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("enum `Status` has no variant `Pending`")),
        "expected unknown enum variant diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_missing_enum_variant_payload_expressions() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

main = () i32 {
    value = Maybe.Some
    0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("missing enum variant payload expression should fail in resolver");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("enum variant `Maybe.Some` requires a payload")),
        "expected missing enum variant payload diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_unexpected_enum_variant_payload_expressions() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

main = () i32 {
    value = Maybe.None(1)
    0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unexpected enum variant payload expression should fail in resolver");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("enum variant `Maybe.None` does not accept a payload")),
        "expected unexpected enum variant payload diagnostic, got {err:?}"
    );
}
