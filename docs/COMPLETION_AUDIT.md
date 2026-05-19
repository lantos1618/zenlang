# Completion Audit

## Objective Restatement

Continue from recovered branch `codex/phase0-1-truth-gates` at recovery commit
`183d140c` from 2026-05-12 08:18:35 UTC. Treat unpushed `/tmp` work after that
commit as lost, reconstruct the durable plan from checked-in docs/tests/commits,
continue TDD-first in small tested commits, preserve the stated design decisions,
and do not assume the broader objective is complete without evidence.

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Status |
|---|---|---|
| Verify repo state | Work continues from git-tracked history; current state is checked with `git status --short --branch`. | satisfied |
| Verify tests before pushing | Required local gates are `cargo fmt --check`, `git diff --check`, focused tests, `cargo clippy -- -D warnings`, `cargo test --lib`, and `cargo test --tests`. | required each slice |
| Keep CI quiet on normal branch pushes | GitHub Actions should stay quiet on normal branch pushes; use PR ready-for-review checks or manual dispatch. | active |
| Recover from `183d140c` | `docs/PHASE_PLAN.md` records the recovery point and current history is built on top of it. | satisfied |
| Reconstruct missing plan | `docs/PHASE_PLAN.md` records design decisions, compressed evidence, current phase, and next slice. | satisfied |
| Identify completed phases | The phase plan and tests summarize Phase 0 truth gates, Phase 1 frontend/C backend, resolver/typechecker work, diagnostics JSON, build graph constraints, stdlib gates, and repo hygiene. | satisfied |
| Continue TDD-first | New behavior slices should add or tighten failing tests before implementation. | ongoing |
| Work in small tested commits | Recent history is split into focused docs, schema, diagnostic, parser, resolver, and cleanup commits. | ongoing |
| Preserve design decisions | See Design Decisions Preserved below and `docs/PHASE_PLAN.md`. | satisfied |
| Avoid completion assumption | Unresolved gaps remain; do not mark the objective complete. | active |

## Design Decisions Preserved

- Sync/Async are real effects, not marker-only types.
- typed allocators remain central to allocation and effect decisions.
- actors live in std first; no actor syntax has been promoted.
- AST/HIR traversal remains tooling/metaprogramming, not core semantics.
- type matching and behavior association remain separate.
- JSON is compiler-owned IR output.
- YAML is human-authored config/spec input.
- `build.zen` is deterministic comptime build graph construction, currently
  constrained by tests rather than open-ended package/link execution.
- `StaticString` is baked program data; allocator-backed `String` is dynamic
  memory and remains allocator-gated.
- Dev UX and Agent UX are explicit roadmap requirements. Zen should grow toward
  MoonBit-style toolchain integration with shared CLI/editor/LSP diagnostics,
  VS Code extension support, language-server target/test discovery,
  agent-readable diagnostics, machine-readable project graph output, and
  structured fix suggestions.

## Compressed Evidence Summary

- Public language docs are separated from status docs. `README.md` is a language
  pitch, `docs/learn_zen_in_y_minutes.md` is a concise tour, and detailed status
  lives here plus `docs/PHASE_PLAN.md`.
- Phase 0 truth gates are guarded by `tests/docs_truth`, including old-spec
  quarantine and public docs consistency.
- Phase 1 frontend and C-backend behavior are guarded by `docs/V1_SPEC.md`,
  `tests/zen`, and integration tests.
- Generic specialization is covered across executable fixtures, JSON golden
  tests, generated-C call/definition scans, imported templates, nested
  `Result<Option<T>, StaticString>`, and multi-specialization cases.
- Resolver/typechecker handoff uses resolver-owned metadata and has focused
  coverage for declaration replay, `generic_struct_constructor_without_type_args_is_error`,
  stale AST avoidance, behavior impl tasks, and bundled semantic passes.
- Diagnostics have stable JSON surfaces with
  `emit_json_diagnostics_command_outputs_machine_readable_errors`,
  `suggested_fixes`, `feature_gate` metadata, removed-`return` fixes, and
  missing bool match arm fixes through
  `emit_json_diagnostics_includes_structured_return_keyword_fix` and
  `emit_json_diagnostics_includes_structured_missing_bool_match_arm_fix`.
- Checked JSON output is pinned by
  `emit_json_ast_marks_semantically_unchecked_sources_that_typed_json_rejects`
  and `emit_json_typed_command_outputs_checked_program`.
- Build graph support is constrained and tested through deterministic build.zen
  parsing, target lowering, dependency ordering, declared host effects,
  library/test/executable target validation, JSON output, and
  `BuildGraphExecutionContext`.
- Repo hygiene is active: `production_rust_files_stay_below_cleanup_threshold`,
  `zen_source_files_stay_below_cleanup_threshold`, owned spelling enum checks,
  old AST node cleanup, and every tracked Rust source file threshold guard keep
  slop from silently returning.
- Gated surfaces are explicit: `Type.implements(Behavior)` covers
  non-generic explicit behavior associations; async scheduler intrinsics are rejected as
  gated not unknown by `async_scheduler_intrinsics_are_rejected_as_gated_not_unknown`;
  `atomic_intrinsics_are_rejected_as_effect_gates`,
  `comptime_type_match_intrinsic_is_rejected_as_gated_not_unknown`,
  `dynamic_string_type_is_rejected_as_allocator_backed_gate`,
  `raw_memory_intrinsics_are_rejected_as_allocator_gates`,
  and `syscall_intrinsics_are_rejected_as_host_effect_gates`.
- Standard library sketches remain visible but guarded, including
  `stdlib/io/mux/uring.zen`, `stdlib/io/net/unix_socket.zen`,
  `stdlib/io/net/socket.zen`, `stdlib/io/files/file.zen`, and
  `stdlib/sys/process/prctl.zen`.

## Unresolved Gaps

- The constrained build-driver is not a full package manager or open-ended link
  system; broader build graph semantics remain gated behind deterministic graph tests.
- Dev UX and Agent UX are planned requirements, not finished product surfaces.
  The VS Code extension, language server, project graph commands, and agent fix
  workflows need dedicated implementation slices.
- Async lowering, allocator-backed dynamic strings, raw memory, byte memory,
  raw pointers, syscalls, and advanced comptime type matching remain gated.
- Generic behavior association and generated association fallback syntax remain
  gated until the behavior solver can model those semantics safely.
- The audit should not be treated as complete until local gates and CI are green
  for the final merged state.

Do not mark the objective complete while these gaps remain.

## Evidence Pointers

- `docs/PHASE_PLAN.md`: current recovery point, design decisions, compressed
  evidence map, current phase, and next slice.
- `docs/V1_SPEC.md`: current language/spec surface.
- `docs/DIAGNOSTICS.md`: stable diagnostics catalog.
- `docs/learn_zen_in_y_minutes.md`: concise public language tour.
- `tests/docs_truth`: documentation, repo hygiene, and status truth gates.
- `tests/integration`: CLI, JSON, diagnostics, build graph, and generated-C
  integration behavior.
- `tests/resolver_phase2.rs`: resolver Phase 2 semantic evidence.
- `tests/zen`: executable language fixtures.
- Git history: detailed implementation evidence that should stay out of compact
  Markdown status docs.
