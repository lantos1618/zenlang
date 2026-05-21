# Completion Audit

## Objective Restatement
Continue from recovered branch `codex/phase0-1-truth-gates` at recovery commit
`183d140c` from 2026-05-12 08:18:35 UTC. Use checked-in docs, tests, and commits
as evidence. Do not mark the objective complete while unresolved gaps remain.
`docs/PHASE_PLAN.md` is the operating plan.

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Status |
|---|---|---|
| Verify repo state | `git status --short --branch`. | required each slice |
| Verify before pushing | `cargo fmt --check`, `git diff --check`, focused tests, `cargo clippy -- -D warnings`, `cargo test --lib`, `cargo test --tests`. | required each slice |
| Keep normal pushes quiet | Branch push Actions should be quiet; PR checks are the ready-review path. | active |
| Preserve durable plan | `docs/PHASE_PLAN.md` records recovery, decisions, evidence, phase, and next slice. | active |

## Design Decisions Preserved
Sync/Async are real effects; typed allocators are central; actors live in std
first; AST/HIR traversal is tooling/metaprogramming; type matching and behavior
association are separate; JSON is compiler-owned IR output; YAML is
human-authored config/spec input; `build.zen` is deterministic comptime graph
construction; `StaticString` is baked program data; allocator-backed `String` is
dynamic memory; `Type.implements(Behavior)` covers non-generic explicit behavior associations.
Dev UX and Agent UX target MoonBit-style toolchain integration,
VS Code extension support, language server workflows, agent-readable diagnostics,
project graph output, structured fix suggestions, and quiet normal branch pushes.

## Compressed Evidence Summary
- Docs split by job: `README.md` pitches, `docs/learn_zen_in_y_minutes.md` teaches, and status lives here plus `docs/PHASE_PLAN.md`.
- Repo shape is guarded by `tests/docs_truth`, `production_rust_files_stay_below_cleanup_threshold`, `zen_source_files_stay_below_cleanup_threshold`, and every tracked Rust source file.
- Frontend/C-backend behavior is guarded by `docs/V1_SPEC.md`, `tests/zen`, generated-C checks, and integration tests.
- Generic specialization covers executable fixtures, JSON golden tests, imports, nested `Result<Option<T>, StaticString>`, generated-C scans, and `generic_struct_constructor_without_type_args_is_error`.
- Diagnostics/JSON evidence includes `emit_json_diagnostics_command_outputs_machine_readable_errors`, `emit_json_diagnostics_includes_structured_return_keyword_fix`, `emit_json_diagnostics_includes_structured_missing_bool_match_arm_fix`, `emit_json_ast_marks_semantically_unchecked_sources_that_typed_json_rejects`, `emit_json_typed_command_outputs_checked_program`, `suggested_fixes`, and `feature_gate`.
- Build graph support is constrained and tested through deterministic build.zen parsing, target lowering, declared host effects, JSON output, and `BuildGraphExecutionContext`; the constrained build-driver is not a package manager, and broader build graph semantics remain gated behind deterministic graph tests.
- Gated surfaces remain explicit: `async_scheduler_intrinsics_are_rejected_as_gated_not_unknown`, `atomic_intrinsics_are_rejected_as_effect_gates`, `comptime_type_match_intrinsic_is_rejected_as_gated_not_unknown`, `dynamic_string_type_is_rejected_as_allocator_backed_gate`, `raw_memory_intrinsics_are_rejected_as_allocator_gates`, and `syscall_intrinsics_are_rejected_as_host_effect_gates`.
- Standard library sketches remain visible but guarded: `stdlib/io/mux/uring.zen`, `stdlib/io/net/unix_socket.zen`, `stdlib/io/net/socket.zen`, `stdlib/io/files/file.zen`, and `stdlib/sys/process/prctl.zen`.

## Unresolved Gaps

Dev UX and Agent UX are planned requirements, not finished product surfaces.
Async lowering, allocator-backed dynamic strings, raw memory, byte memory, raw
pointers, syscalls, advanced comptime type matching, generic behavior association,
generated association fallback syntax, and broad package/link build-driver
behavior remain gated.

## Evidence Pointers

Use `docs/PHASE_PLAN.md`, `docs/V1_SPEC.md`, `docs/DIAGNOSTICS.md`,
`docs/learn_zen_in_y_minutes.md`, `tests/docs_truth`, `tests/integration`,
`tests/resolver_phase2.rs`, `tests/zen`, and git history. Keep implementation
detail in tests, fixtures, and commits instead of expanding compact Markdown.
