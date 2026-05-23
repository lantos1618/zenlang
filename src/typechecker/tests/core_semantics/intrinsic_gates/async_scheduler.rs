use super::*;

#[test]
fn async_scheduler_intrinsics_are_rejected_as_gated_not_unknown() {
    let program = parse_program(
        r#"
main = () void {
    @builtin.async_enqueue(1)
    @builtin.async_yield()
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc.check_program(&program).expect_err(
        "async scheduler intrinsics should stay gated until effect checking and task lowering exist",
    );

    for expected in ["async task enqueue is gated", "async yield is gated"] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown function `@builtin.async_")),
        "async scheduler gates should not be reported as ordinary unknown builtins, got {err:?}"
    );
}
