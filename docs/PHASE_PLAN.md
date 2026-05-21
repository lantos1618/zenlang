# Phase Plan

## Recovery Point
Recovered branch `codex/phase0-1-truth-gates` at commit `183d140c`
(2026-05-12 08:18:35 UTC). Treat unpushed `/tmp` work after that as lost;
continue from checked-in docs, tests, and commits only.

## Design Decisions To Preserve
- Sync/Async are real effects; typed allocators drive allocation/effect rules.
- actors live in std first; no actor syntax is v1-stable yet.
- AST/HIR traversal is tooling/metaprogramming, not core semantics.
- type matching and behavior association are separate mechanisms.
- JSON is compiler-owned IR output; YAML is human-authored config/spec input.
- build.zen is deterministic comptime build graph construction.
- `StaticString` is baked program data; allocator-backed `String` is dynamic.
- `Type.implements(Behavior)` covers non-generic explicit behavior associations.
- Dev UX and Agent UX are product requirements, not polish.

## Dev UX And Agent UX Track
MoonBit-style toolchain integration is the benchmark: compiler, build graph, package surface, language server, VS Code extension, web/editor entry point, and machine-readable outputs should feel coherent.
Required Dev UX: syntax/semantic diagnostics, go-to-definition, hover, completion, formatting, run/test code lenses, target selection, language server restart, compiler version display, local toolchain validation, and `zen lsp`.
Required Agent UX: agent-readable diagnostics with stable codes, spans, related locations, suggested_fixes, feature_gate metadata, CLI/editor-aligned JSON, Machine-readable project graph and symbol graph output, deterministic quiet commands, structured fix suggestions, retrieval-friendly docs, and quiet normal branch pushes.

## Compressed Evidence Map
This is a capability index, not a changelog; granular evidence belongs in tests,
golden fixtures, and git history:

- Phase 0 truth gates: README, contributor docs, stdlib, CI, release, old-spec quarantine, and docs shape are guarded by `tests/docs_truth`.
- Phase 1 frontend and C-backend baseline: syntax and C execution are covered by `docs/V1_SPEC.md`, `tests/zen`, integration tests, and generated-C checks.
- generic specialization: functions, structs, enums, methods, recursive worklists, imports, nested `Result<Option<T>, StaticString>`, and generated-C consistency are covered by executable and JSON-golden integration tests.
- resolver/typechecker replay: resolver-owned metadata, callable signatures, behavior impl passes, stale AST protection, `generic_struct_constructor_without_type_args_is_error`, and generic arity diagnostics are covered by `tests/resolver_phase2.rs`, typechecker unit tests, and integration diagnostics.
- diagnostics JSON: `emit_json_diagnostics_command_outputs_machine_readable_errors`, `emit_json_diagnostics_includes_structured_return_keyword_fix`, and `emit_json_diagnostics_includes_structured_missing_bool_match_arm_fix` pin stable diagnostics with suggested_fixes and feature_gate data.
- Typed/HIR/MIR JSON: `emit_json_ast_marks_semantically_unchecked_sources_that_typed_json_rejects`
  and `emit_json_typed_command_outputs_checked_program` guard checked output.
- build graph: deterministic build.zen graph behavior is guarded by parser, lowering, JSON, CLI tests, and `deterministic_build_graph_creates_one_executable_target`.
- Gated effects/intrinsics: `async_scheduler_intrinsics_are_rejected_as_gated_not_unknown`, `atomic_intrinsics_are_rejected_as_effect_gates`, `comptime_type_match_intrinsic_is_rejected_as_gated_not_unknown`, `dynamic_string_type_is_rejected_as_allocator_backed_gate`, `raw_memory_intrinsics_are_rejected_as_allocator_gates`, and `syscall_intrinsics_are_rejected_as_host_effect_gates`.
- repo hygiene: file-size tests, owned spelling enums, syntax cleanup tests, and docs-truth caps prevent large-file and status-doc regressions.

## Current Phase
Phase 5 is in evidence-hardening and cleanup. The generic specialization
surfaces are implemented; continue closing proof gaps, keeping generated C
consistent, and preventing large-file/slop regressions.

## Phase 5 Acceptance Evidence
generic enum specialization, generic method specialization, worklist monomorphization,
generated-C call/definition consistency, and generic arity, inference, and bound diagnostics are covered by executable fixtures,
typed/HIR/MIR JSON golden tests, generated-C scans, E5000/E5001/E5002/E6004
diagnostics, `compile_to_c_with_generated_call_check`, and
`undefined_generated_c_calls`.

Non-Phase-5 gaps remain Dev UX, Agent UX, full LSP/editor workflows,
allocator-backed dynamic strings, Sync/Async lowering, raw memory semantics,
advanced comptime type matching, and broad package/link build-driver behavior.

## Next Small Slice
Pick one oversized Rust file, add or tighten a focused repo-hygiene/test guard, move one coherent responsibility into a focused module, run local gates, confirm normal branch-push Actions stay quiet, open a ready PR, and merge only when PR checks pass.

## Detailed Evidence References
Use `docs/V1_SPEC.md`, `docs/DIAGNOSTICS.md`,
`docs/learn_zen_in_y_minutes.md`, `docs/COMPLETION_AUDIT.md`,
`tests/docs_truth`, `tests/integration`, `tests/resolver_phase2.rs`,
`tests/zen`, and git history. Keep implementation detail in tests, fixtures,
and commits instead of expanding status Markdown.
