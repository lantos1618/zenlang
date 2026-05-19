use super::super::*;

#[test]
fn v1_spec_records_feature_matrix_gates_and_ux_requirements() {
    let spec = read("docs/V1_SPEC.md");

    for required in [
        "Status: v1 draft",
        "Feature Matrix",
        "implemented",
        "gated",
        "experimental",
        "removed",
        "Sync/Async effects",
        "async_scheduler_intrinsics_are_rejected_as_gated_not_unknown",
        "stdlib_async_runtime_import_is_gated_before_loading_sketch",
        "module_graph_gates_stdlib_async_runtime_import_before_loading_sketch",
        "emit_json_diagnostics_async_runtime_import_gate_schema_matches_golden",
        "stdlib_sync_runtime_import_is_gated_before_loading_sketch",
        "module_graph_gates_stdlib_sync_runtime_import_before_loading_sketch",
        "emit_json_diagnostics_sync_runtime_import_gate_schema_matches_golden",
        "async task enqueue",
        "async yield",
        "atomic_intrinsics_are_rejected_as_effect_gates",
        "@builtin.atomic_load",
        "@builtin.atomic_store",
        "@builtin.atomic_add",
        "@builtin.atomic_sub",
        "@builtin.atomic_cas",
        "@builtin.atomic_xchg",
        "@builtin.fence",
        "Typed allocators",
        "raw_memory_intrinsics_are_rejected_as_allocator_gates",
        "byte_memory_intrinsics_are_rejected_as_allocator_gates",
        "sync_and_async_typed_allocator_modes_are_rejected_as_gated_not_unknown",
        "stdlib_allocator_import_is_gated_before_loading_sketch",
        "module_graph_gates_stdlib_allocator_import_before_loading_sketch",
        "emit_json_diagnostics_allocator_import_gate_schema_matches_golden",
        "@builtin.raw_allocate",
        "@builtin.raw_deallocate",
        "@builtin.raw_reallocate",
        "@builtin.memcpy",
        "@builtin.memmove",
        "@builtin.memset",
        "@builtin.memcmp",
        "Type matching",
        "Behavior association",
        "AST traversal",
        "`Channel` remains an experimental stdlib channel",
        "semantic_status: \"unchecked\"",
        "typed JSON is explicitly marked checked",
        "diagnostics JSON is explicitly",
        "docs/DIAGNOSTICS.md",
        "JSON-stable public diagnostic codes",
        "broader diagnostic-code coverage is still required",
        "context.kind = \"feature_gate\"",
        "Developer UX and Agent UX",
        "product requirements, not polish",
        "MoonBit-style toolchain integration",
        "VS Code extension remains a constrained editor wrapper",
        "`zen lsp` remains gated",
        "Agent-readable diagnostics",
        "machine-readable project graph",
        "structured fix suggestions",
        "quiet deterministic commands",
        "must not advertise unsupported language-server binaries",
        "Type.implements(Behavior)",
        "non-generic explicit behavior associations",
        "Actors in std",
        "bare_actor_framework_types_are_rejected_as_gated_not_unknown",
        "stdlib_actor_framework_import_is_gated_before_loading_sketch",
        "module_graph_gates_stdlib_actor_framework_import_before_loading_sketch",
        "emit_json_diagnostics_actor_import_gate_schema_matches_golden",
        "JSON/YAML IR boundaries",
        "comptime_type_match_intrinsic_is_rejected_as_gated_not_unknown",
        "primitive_and_enum_type_match_intrinsics_are_rejected_as_gated_not_unknown",
        "comptime type matching",
        "gated until typed metadata and derive lowering exist",
        "Ownership and raw pointer operations",
        "raw_pointer_intrinsics_are_rejected_as_ownership_gates",
        "@builtin.gep",
        "@builtin.gep_struct",
        "@builtin.raw_ptr_cast",
        "@builtin.ptr_to_int",
        "@builtin.int_to_ptr",
        "@builtin.load<T>",
        "@builtin.store<T>",
        "Host syscalls",
        "syscall_intrinsics_are_rejected_as_host_effect_gates",
        "@builtin.syscall0",
        "@builtin.syscall6",
    ] {
        assert!(
            spec.contains(required),
            "docs/V1_SPEC.md is missing feature matrix or UX requirement: {required}"
        );
    }

    assert!(
        !spec.contains("| Strict resolver, symbol IDs, privacy | gated |"),
        "docs/V1_SPEC.md should not describe implemented resolver/module privacy evidence as gated"
    );
    assert!(
        !spec.contains("`ActorRef`, `Mailbox`, `Channel`, and `Supervisor`"),
        "docs/V1_SPEC.md should not imply Channel is a globally gated actor builtin"
    );
    assert!(
        !spec.contains("sync and async allocator tests still required"),
        "docs/V1_SPEC.md should not keep stale allocator-test backlog wording after Sync/Async allocator gate coverage exists"
    );
}
