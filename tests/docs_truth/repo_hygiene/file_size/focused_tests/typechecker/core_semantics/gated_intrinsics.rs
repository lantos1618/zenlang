use super::*;

#[test]
fn intrinsic_gate_tests_stay_split_by_effect_family() {
    let root = read("src/typechecker/tests/core_semantics/intrinsic_gates.rs");
    let async_scheduler =
        read("src/typechecker/tests/core_semantics/intrinsic_gates/async_scheduler.rs");
    let atomics = read("src/typechecker/tests/core_semantics/intrinsic_gates/atomics.rs");
    let raw_memory = read("src/typechecker/tests/core_semantics/intrinsic_gates/raw_memory.rs");
    let raw_pointers = read("src/typechecker/tests/core_semantics/intrinsic_gates/raw_pointers.rs");
    let syscalls = read("src/typechecker/tests/core_semantics/intrinsic_gates/syscalls.rs");

    assert!(
        root.lines().count() < 80,
        "intrinsic_gates.rs should only route focused intrinsic gate tests"
    );
    for module in [
        "mod async_scheduler;",
        "mod atomics;",
        "mod raw_memory;",
        "mod raw_pointers;",
        "mod syscalls;",
        "mod type_match;",
    ] {
        assert!(
            root.contains(module),
            "intrinsic_gates.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn async_scheduler_intrinsics_are_rejected_as_gated_not_unknown"),
        "async scheduler gate tests should live in async_scheduler.rs"
    );
    assert!(
        async_scheduler.contains("fn async_scheduler_intrinsics_are_rejected_as_gated_not_unknown"),
        "async_scheduler.rs should cover async scheduler gates"
    );
    assert!(
        raw_memory.contains("fn byte_memory_intrinsics_are_rejected_as_allocator_gates"),
        "raw_memory.rs should cover allocator-backed byte memory gates"
    );
    assert!(
        raw_pointers.contains("fn raw_pointer_intrinsics_are_rejected_as_ownership_gates"),
        "raw_pointers.rs should cover raw pointer ownership gates"
    );
    assert!(
        atomics.contains("fn atomic_intrinsics_are_rejected_as_effect_gates"),
        "atomics.rs should cover atomic effect gates"
    );
    assert!(
        syscalls.contains("fn syscall_intrinsics_are_rejected_as_host_effect_gates"),
        "syscalls.rs should cover syscall host-effect gates"
    );
}
