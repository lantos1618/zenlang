use super::super::*;

#[test]
fn completion_audit_records_recovery_objective_evidence_and_gaps() {
    let audit = read("docs/COMPLETION_AUDIT.md");

    for required in [
        "Objective Restatement",
        "Prompt-To-Artifact Checklist",
        "183d140c",
        "docs/PHASE_PLAN.md",
        "Compressed Evidence Summary",
        "cargo fmt --check",
        "cargo clippy -- -D warnings",
        "cargo test --lib",
        "cargo test --tests",
        "Design Decisions Preserved",
        "Unresolved Gaps",
        "Evidence Pointers",
        "Dev UX and Agent UX",
        "MoonBit-style toolchain integration",
        "VS Code extension",
        "agent-readable diagnostics",
        "broader build graph semantics remain gated behind deterministic graph tests",
        "constrained build-driver",
        "Do not mark the objective complete",
        "production_rust_files_stay_below_cleanup_threshold",
        "zen_source_files_stay_below_cleanup_threshold",
        "stdlib/io/mux/uring.zen",
        "every tracked Rust source file",
        "BuildGraphExecutionContext",
        "emit_json_ast_marks_semantically_unchecked_sources_that_typed_json_rejects",
        "emit_json_typed_command_outputs_checked_program",
        "emit_json_diagnostics_command_outputs_machine_readable_errors",
        "emit_json_diagnostics_includes_structured_return_keyword_fix",
        "emit_json_diagnostics_includes_structured_missing_bool_match_arm_fix",
        "suggested_fixes",
        "feature_gate",
        "Type.implements(Behavior)",
        "non-generic explicit behavior associations",
        "generic_struct_constructor_without_type_args_is_error",
        "async_scheduler_intrinsics_are_rejected_as_gated_not_unknown",
        "atomic_intrinsics_are_rejected_as_effect_gates",
        "comptime_type_match_intrinsic_is_rejected_as_gated_not_unknown",
        "dynamic_string_type_is_rejected_as_allocator_backed_gate",
        "raw_memory_intrinsics_are_rejected_as_allocator_gates",
        "syscall_intrinsics_are_rejected_as_host_effect_gates",
        "StaticString",
        "allocator-backed `String`",
        "docs/V1_SPEC.md",
        "docs/DIAGNOSTICS.md",
        "docs/learn_zen_in_y_minutes.md",
        "tests/docs_truth",
        "tests/integration",
        "tests/resolver_phase2.rs",
        "tests/zen",
        "stdlib/io/net/unix_socket.zen",
        "stdlib/io/net/socket.zen",
        "stdlib/io/files/file.zen",
        "stdlib/sys/process/prctl.zen",
    ]
    .into_iter()
    {
        assert!(
            audit.contains(required),
            "docs/COMPLETION_AUDIT.md is missing audit evidence text: {required}"
        );
    }

    assert!(
        audit.lines().count() <= 120,
        "docs/COMPLETION_AUDIT.md should stay compact; move granular evidence to tests or git history"
    );

    assert!(
        !audit.contains("Generic behavior inheritance in `.extends` is still explicitly gated"),
        "docs/COMPLETION_AUDIT.md still claims generic behavior inheritance is gated"
    );
}
