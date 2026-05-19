# Completion Audit

## Objective Restatement

Continue from recovered branch `codex/phase0-1-truth-gates` at recovery commit
`183d140c` from 2026-05-12 08:18:35 UTC. Treat unpushed `/tmp` work after that
commit as lost, reconstruct the durable plan from checked-in docs/tests/commits,
continue TDD-first in small tested commits, preserve design decisions, and do
not assume completion without evidence.

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Status |
|---|---|---|
| Verify repo state | Check `git status --short --branch`. | required each slice |
| Verify tests before pushing | Run `cargo fmt --check`, `git diff --check`, focused tests, `cargo clippy -- -D warnings`, `cargo test --lib`, and `cargo test --tests`. | required each slice |
| Keep CI quiet on normal branch pushes | Normal branch pushes should not fan out GitHub Actions; PR checks are the ready-review path. | active |
| Recover from `183d140c` | `docs/PHASE_PLAN.md` records the recovery point. | satisfied |
| Reconstruct missing plan | `docs/PHASE_PLAN.md` records design decisions, compressed evidence, current phase, and next slice. | satisfied |
| Continue TDD-first | New language/compiler behavior starts with a failing or tightened test. | ongoing |
| Work in small tested commits | Recent history is split into focused docs, schema, diagnostic, parser, resolver, and cleanup commits. | ongoing |
| Preserve design decisions | See below and `docs/PHASE_PLAN.md`. | satisfied |
| Avoid completion assumption | Unresolved gaps remain. | active |

## Design Decisions Preserved

Sync/Async are real effects; typed allocators remain central; actors live in std
first; AST/HIR traversal remains tooling/metaprogramming; type matching and
behavior association remain separate; JSON is compiler-owned IR output; YAML is
human-authored config/spec input; `build.zen` is deterministic comptime build
graph construction; `StaticString` is baked program data; allocator-backed `String`
is dynamic memory; `Type.implements(Behavior)` covers non-generic explicit behavior associations;
Dev UX and Agent UX target MoonBit-style toolchain integration with a VS Code extension,
language server, agent-readable diagnostics, project graph output, and
structured fix suggestions.

## Compressed Evidence Summary

- Public docs are split by job: `README.md` pitches the language,
  `docs/learn_zen_in_y_minutes.md` teaches it, and status lives here plus
  `docs/PHASE_PLAN.md`.
- Phase 0 truth gates and repo shape are guarded by `tests/docs_truth`,
  including `production_rust_files_stay_below_cleanup_threshold`,
  `zen_source_files_stay_below_cleanup_threshold`, and every tracked Rust source file
  threshold.
- Phase 1 frontend/C-backend behavior is guarded by `docs/V1_SPEC.md`,
  `tests/zen`, generated-C checks, and integration tests.
- Generic specialization covers executable fixtures, JSON golden tests,
  imported templates, nested `Result<Option<T>, StaticString>`, and
  generated-C definition/call scans.
- Resolver/typechecker handoff uses resolver-owned metadata with coverage for
  declaration replay, `generic_struct_constructor_without_type_args_is_error`,
  stale AST avoidance, and behavior impl passes.
- Diagnostics have stable JSON surfaces through
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
- Standard library sketches remain visible but guarded, including
  `stdlib/io/mux/uring.zen`, `stdlib/io/net/unix_socket.zen`,
  `stdlib/io/net/socket.zen`, `stdlib/io/files/file.zen`, and
  `stdlib/sys/process/prctl.zen`.

## Unresolved Gaps

- The constrained build-driver is not a full package manager or open-ended link
  system; broader build graph semantics remain gated behind deterministic graph tests.
- Dev UX and Agent UX are planned requirements, not finished product surfaces.
- Async lowering, allocator-backed dynamic strings, raw memory, byte memory,
  raw pointers, syscalls, and advanced comptime type matching remain gated.
- Generic behavior association and generated association fallback syntax remain
  gated until solver semantics exist.

Do not mark the objective complete while these gaps remain.

## Evidence Pointers

- `docs/PHASE_PLAN.md`: recovery point, design decisions, evidence map, phase,
  and next slice.
- `docs/V1_SPEC.md`: current language/spec surface.
- `docs/DIAGNOSTICS.md`: stable diagnostics catalog.
- `docs/learn_zen_in_y_minutes.md`: concise public language tour.
- `tests/docs_truth`: documentation, repo hygiene, and status truth gates.
- `tests/integration`: CLI, JSON, diagnostics, build graph, and generated-C.
- `tests/resolver_phase2.rs`: resolver Phase 2 semantic evidence.
- `tests/zen`: executable language fixtures.
- Git history: implementation detail that should stay out of compact Markdown.
