use super::super::*;

#[test]
fn phase_plan_records_recovered_progress_and_next_slice() {
    let plan = read("docs/PHASE_PLAN.md");

    for required in [
        "Recovery Point",
        "183d140c",
        "Design Decisions To Preserve",
        "Dev UX And Agent UX Track",
        "Compressed Evidence Map",
        "Current Phase",
        "Phase 5 Acceptance Evidence",
        "generic enum specialization",
        "generic method specialization",
        "worklist monomorphization",
        "generated-C call/definition consistency",
        "generic arity, inference, and bound diagnostics",
        "Next Small Slice",
        "Detailed Evidence References",
        "Sync/Async are real effects",
        "typed allocators",
        "actors live in std first",
        "AST/HIR traversal is tooling/metaprogramming",
        "type matching and behavior association are separate",
        "JSON is compiler-owned IR output",
        "YAML is human-authored config/spec input",
        "build.zen is deterministic comptime build graph",
        "MoonBit-style toolchain integration",
        "VS Code extension",
        "language server",
        "agent-readable diagnostics",
        "Machine-readable project graph",
        "structured fix suggestions",
        "Phase 0 truth gates",
        "Phase 1 frontend and C-backend baseline",
        "generic specialization",
        "resolver/typechecker replay",
        "build graph",
        "diagnostics JSON",
        "repo hygiene",
        "docs/V1_SPEC.md",
        "docs/DIAGNOSTICS.md",
        "docs/learn_zen_in_y_minutes.md",
        "tests/docs_truth",
        "tests/integration",
        "tests/resolver_phase2.rs",
        "tests/zen",
        "deterministic_build_graph_creates_one_executable_target",
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
    ]
    .into_iter()
    {
        assert!(
            plan.contains(required),
            "docs/PHASE_PLAN.md is missing durable plan text: {required}"
        );
    }

    assert!(
        plan.lines().count() <= 115,
        "docs/PHASE_PLAN.md should stay compact; move granular evidence to tests or git history"
    );

    assert!(
        !plan.contains("Generic behavior inheritance in `.extends` is still explicitly gated"),
        "docs/PHASE_PLAN.md still claims generic behavior inheritance is gated"
    );
}
