use super::*;

#[test]
fn resolver_gates_generated_behavior_derive_association() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}

Point: { x: i32 }

Point.derive(Json)
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("generated derive associations should stay gated");

    assert!(
        err.iter().any(|d| {
            d.message
                .contains("generated behavior association `Type.derive(...)` is gated")
        }),
        "expected generated behavior association gate, got {err:?}"
    );
}
