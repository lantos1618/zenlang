use super::*;

#[test]
fn raw_pointer_intrinsics_are_rejected_as_ownership_gates() {
    let program = parse_program(
        r#"
main = () void {
    @builtin.gep(0, 1)
    @builtin.raw_ptr_offset(0, 1)
    @builtin.gep_struct(0, 1)
    @builtin.raw_ptr_cast(0)
    @builtin.ptr_to_int(0)
    @builtin.int_to_ptr(0)
    @builtin.load<i32>(0)
    @builtin.store<i32>(0, 1)
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("raw pointer intrinsics should stay gated until ownership semantics exist");

    let pointer_offset_gate_count = err
        .iter()
        .filter(|diagnostic| diagnostic.message.contains("raw pointer offset is gated"))
        .count();
    assert_eq!(
        pointer_offset_gate_count, 2,
        "expected exactly gep and raw_ptr_offset ownership gates, got {err:?}"
    );

    for expected in [
        "raw struct pointer offset is gated",
        "raw pointer cast is gated",
        "raw pointer to integer conversion is gated",
        "integer to raw pointer conversion is gated",
        "raw pointer load is gated",
        "raw pointer store is gated",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter().all(|d| {
            !d.message.contains("unknown function `@builtin.")
                && !d.message.contains("does not accept type arguments")
        }),
        "raw pointer gates should not be reported as ordinary unknown/generic builtin failures, got {err:?}"
    );
}
