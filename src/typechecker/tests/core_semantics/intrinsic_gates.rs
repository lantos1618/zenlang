use super::*;

mod type_match;

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
    assert!(
        pointer_offset_gate_count >= 2,
        "expected gep and raw_ptr_offset ownership gates, got {err:?}"
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

#[test]
fn atomic_intrinsics_are_rejected_as_effect_gates() {
    let program = parse_program(
        r#"
main = () void {
    @builtin.atomic_load(0)
    @builtin.atomic_store(0, 1)
    @builtin.atomic_add(0, 1)
    @builtin.atomic_sub(0, 1)
    @builtin.atomic_cas(0, 1, 2)
    @builtin.atomic_xchg(0, 1)
    @builtin.fence()
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("atomic intrinsics should stay gated until memory-order semantics exist");

    for expected in [
        "atomic load is gated",
        "atomic store is gated",
        "atomic add is gated",
        "atomic subtract is gated",
        "atomic compare-and-swap is gated",
        "atomic exchange is gated",
        "atomic fence is gated",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown function `@builtin.atomic_")),
        "atomic gates should not be reported as ordinary unknown builtins, got {err:?}"
    );
}

#[test]
fn syscall_intrinsics_are_rejected_as_host_effect_gates() {
    let program = parse_program(
        r#"
main = () void {
    @builtin.syscall0(1)
    @builtin.syscall1(1, 2)
    @builtin.syscall2(1, 2, 3)
    @builtin.syscall3(1, 2, 3, 4)
    @builtin.syscall4(1, 2, 3, 4, 5)
    @builtin.syscall5(1, 2, 3, 4, 5, 6)
    @builtin.syscall6(1, 2, 3, 4, 5, 6, 7)
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("syscall intrinsics should stay gated until host effects exist");

    for expected in [
        "syscall0 is gated",
        "syscall1 is gated",
        "syscall2 is gated",
        "syscall3 is gated",
        "syscall4 is gated",
        "syscall5 is gated",
        "syscall6 is gated",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown function `@builtin.syscall")),
        "syscall gates should not be reported as ordinary unknown builtins, got {err:?}"
    );
}
