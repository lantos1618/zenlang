use super::*;

#[test]
fn raw_memory_intrinsics_are_rejected_as_allocator_gates() {
    let program = parse_program(
        r#"
main = () void {
    ptr = @builtin.raw_allocate(8)
    @builtin.raw_deallocate(ptr, 8)
    @builtin.raw_reallocate(ptr, 8, 16)
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("raw memory intrinsics should stay gated until allocator semantics exist");

    for expected in [
        "raw allocation is gated",
        "raw deallocation is gated",
        "raw reallocation is gated",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown function `@builtin.raw_")),
        "raw memory gates should not be reported as ordinary unknown builtins, got {err:?}"
    );
}
