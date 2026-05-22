use super::*;

#[test]
fn resolver_rejects_duplicate_behavior_method_names() {
    let program = parse_program(
        r#"
Serializable: behavior {
    encode: (Self) StaticString
    encode: (Self, i32) StaticString
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate behavior method names should fail in resolver");

    assert!(
        err.iter().any(|d| {
            d.message
                .contains("duplicate behavior method `encode` in `Serializable`")
        }),
        "expected duplicate behavior method diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_duplicate_signature_parameter_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (value: Self, value: Self) StaticString
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate behavior method parameter names should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate parameter `value`")),
        "expected duplicate behavior method parameter diagnostic, got {err:?}"
    );
}
