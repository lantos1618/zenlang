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

#[test]
fn byte_memory_intrinsics_are_rejected_as_allocator_gates() {
    let program = parse_program(
        r#"
main = () void {
    @builtin.memset(0, 0, 8)
    @builtin.memcpy(0, 0, 8)
    @builtin.memmove(0, 0, 8)
    @builtin.memcmp(0, 0, 8)
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("byte memory intrinsics should stay gated until allocator semantics exist");

    for expected in [
        "raw memory set is gated",
        "raw memory copy is gated",
        "raw memory move is gated",
        "raw memory compare is gated",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter().all(|d| {
            !d.message.contains("unknown function `@builtin.mem")
                && !d.message.contains("unknown function `@builtin.raw_")
        }),
        "byte memory gates should not be reported as ordinary unknown builtins, got {err:?}"
    );
}
