# Completion Audit

## Objective Restatement

Continue from recovered branch `codex/phase0-1-truth-gates` at recovery commit
`183d140c` from 2026-05-12 08:18:35 UTC. Reconstruct durable work from checked-in
docs/tests/commits, keep slices TDD-first, and do not assume completion without
evidence. `docs/PHASE_PLAN.md` is the operating plan.

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Status |
|---|---|---|
| Verify repo state | `git status --short --branch`. | required each slice |
| Verify tests before pushing | `cargo fmt --check`, `git diff --check`, focused tests, `cargo clippy -- -D warnings`, `cargo test --lib`, `cargo test --tests`. | required each slice |
| Keep CI quiet on normal branch pushes | Branch push Actions should be quiet; PR checks are the ready-review path. | active |
| Recover/reconstruct plan | `docs/PHASE_PLAN.md` records recovery, decisions, evidence, phase, and next slice. | satisfied |
| Preserve decisions and avoid completion assumption | See preserved decisions and unresolved gaps below. | active |

## Design Decisions Preserved

Sync/Async are real effects; typed allocators are central; actors live in std first;
AST/HIR traversal is tooling/metaprogramming; type matching and behavior association
are separate; JSON is compiler-owned IR output; YAML is human-authored config/spec
input; `build.zen` is deterministic comptime build graph construction; `StaticString`
is baked program data; allocator-backed `String` is dynamic memory;
`Type.implements(Behavior)` covers non-generic explicit behavior associations. Dev
UX and Agent UX target MoonBit-style toolchain integration with a VS Code extension,
language server, agent-readable diagnostics, project graph output, and structured fix
suggestions.

## Compressed Evidence Summary

- Public docs are split by job: `README.md` pitches, `docs/learn_zen_in_y_minutes.md` teaches, and status lives here plus `docs/PHASE_PLAN.md`.
- Phase 0 and repo shape are guarded by `tests/docs_truth`, including
  `production_rust_files_stay_below_cleanup_threshold`,
  `zen_source_files_stay_below_cleanup_threshold`, and every tracked Rust source file.
- Frontend/C-backend behavior is guarded by `docs/V1_SPEC.md`, `tests/zen`,
  generated-C checks, and integration tests.
- Generic specialization covers executable fixtures, JSON golden tests, imports, nested `Result<Option<T>, StaticString>`, generated-C scans, and `generic_struct_constructor_without_type_args_is_error`.
- Resolver/typechecker handoff uses resolver-owned metadata with declaration replay, stale AST avoidance, and behavior impl passes.
- Diagnostics have stable JSON through
  `emit_json_diagnostics_command_outputs_machine_readable_errors`,
  `emit_json_diagnostics_includes_structured_return_keyword_fix`,
  `emit_json_diagnostics_includes_structured_missing_bool_match_arm_fix`,
  suggested_fixes, and feature_gate metadata.
- Checked JSON output is pinned by
  `emit_json_ast_marks_semantically_unchecked_sources_that_typed_json_rejects`
  and `emit_json_typed_command_outputs_checked_program`.
- Build graph support is constrained and tested through deterministic build.zen
  parsing, target lowering, declared host effects, JSON output, and
  `BuildGraphExecutionContext`.
- Gated surfaces are explicit:
  `async_scheduler_intrinsics_are_rejected_as_gated_not_unknown`,
  `atomic_intrinsics_are_rejected_as_effect_gates`,
  `comptime_type_match_intrinsic_is_rejected_as_gated_not_unknown`,
  `dynamic_string_type_is_rejected_as_allocator_backed_gate`,
  `raw_memory_intrinsics_are_rejected_as_allocator_gates`, and
  `syscall_intrinsics_are_rejected_as_host_effect_gates`.
- Standard library sketches remain visible but guarded: `stdlib/io/mux/uring.zen`, `stdlib/io/net/unix_socket.zen`, `stdlib/io/net/socket.zen`, `stdlib/io/files/file.zen`, and `stdlib/sys/process/prctl.zen`.

## Unresolved Gaps

- The constrained build-driver is not a package manager or open-ended link
  system; broader build graph semantics remain gated behind deterministic graph tests.
- Dev UX and Agent UX are planned requirements, not finished product surfaces.
- Async lowering, allocator-backed dynamic strings, raw memory, byte memory,
  raw pointers, syscalls, and advanced comptime type matching remain gated.
- Generic behavior association and generated association fallback syntax remain
  gated until solver semantics exist.

Do not mark the objective complete while these gaps remain.

## Evidence Pointers

Use `docs/PHASE_PLAN.md`, `docs/V1_SPEC.md`, `docs/DIAGNOSTICS.md`,
`docs/learn_zen_in_y_minutes.md`, `tests/docs_truth`, `tests/integration`,
`tests/resolver_phase2.rs`, `tests/zen`, and git history. Keep implementation
detail in tests, fixtures, and commits instead of expanding compact Markdown.
