use super::*;

#[test]
fn enum_payload_mutation_intrinsic_is_rejected_as_layout_gate() {
    let program = parse_program(
        r#"
main = () void {
    @builtin.set_payload(0, 0)
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("enum payload mutation should stay gated until payload layout sizes exist");

    assert!(
        err.iter()
            .any(|d| d.message.contains("enum payload mutation is gated")),
        "expected enum payload mutation gate diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown function `@builtin.set_payload`")),
        "enum payload mutation gate should not be reported as an ordinary unknown builtin, got {err:?}"
    );
}
