# Phase Plan

## Recovery Point

Recovered branch: `codex/phase0-1-truth-gates`.

Recovery commit: `183d140c` from 2026-05-12 08:18:35 UTC.

Any unpushed `/tmp` work after that commit is treated as lost. Continue from
checked-in docs, tests, and commits only.

## Design Decisions To Preserve

- Sync/Async are real effects, not marker-only types.
- typed allocators are central to allocation and effect decisions.
- actors live in std first; no actor syntax is v1-stable yet.
- AST/HIR traversal is tooling/metaprogramming, not core semantics.
- type matching and behavior association are separate mechanisms.
- JSON is compiler-owned IR output.
- YAML is human-authored config/spec input.
- build.zen is deterministic comptime build graph construction.
- `StaticString` is baked program data; allocator-backed `String` remains a
  separate dynamic type and must not be blurred in docs or diagnostics.
- `Type.implements(Behavior)` stays as the explicit non-generic behavior
  association spelling until a solver can handle more advanced forms.
- Dev UX and Agent UX are product requirements, not polish.

## Dev UX And Agent UX Track

MoonBit-style toolchain integration is the benchmark: one coherent compiler,
build system, package graph, language server, VS Code extension, web/editor
entry point, and machine-readable surface area rather than disconnected tools.

Dev UX requirements: VS Code syntax, semantic diagnostics, go-to-definition,
hover, completion, formatting, run/test code lenses, target selection, language
server restart, compiler version display, local toolchain validation, and
`zen lsp` backed by the CLI parser, resolver, typechecker, build graph, and
diagnostics. Editor build graph integration must cover target/test discovery,
dependency inspection, run/test/build, declared host effects, deterministic
errors, and structured fix suggestions for missing match arms, generic arity
mistakes, removed syntax such as `return`, gated features, missing imports, and
common type mismatches.

Agent UX requirements: agent-readable diagnostics with stable codes, spans,
related locations, suggested_fixes, feature_gate metadata, CLI/editor-aligned
JSON. Machine-readable project graph and symbol graph output, deterministic
quiet commands for `zen check`, `zen test`, `zen emit-json`, build graph
inspection, formatting, future fix-application workflows, retrieval-friendly
canonical docs, and quiet branch-push CI.

## Compressed Evidence Map

This section records current capability groups, not every historical test name.
Granular evidence belongs in tests, golden fixtures, and git history.

- Phase 0 truth gates: README, contributor, stdlib, CI, release, old-spec
  quarantine, and docs shape are guarded by `tests/docs_truth`.
- Phase 1 frontend and C-backend baseline: supported syntax and C execution
  behavior are documented in `docs/V1_SPEC.md` and covered by `tests/zen` plus
  integration tests.
- generic specialization: generic functions, structs, enums, methods,
  recursive worklists, imported generic templates, nested `Result<Option<T>,
  StaticString>` cases, and generated-C call/definition consistency are covered
  by executable and JSON-golden integration tests.
- resolver/typechecker replay: resolver-owned declaration metadata, callable
  signatures, behavior impl passes, stale AST protection, and generic arity
  diagnostics are covered by `tests/resolver_phase2.rs`, typechecker unit
  tests, and integration diagnostics. Representative tests include
  `generic_struct_constructor_without_type_args_is_error` and the
  non-generic explicit behavior associations around `Type.implements(Behavior)`.
- Behavior and type matching boundaries: type matching remains separate from
  behavior association; generic association targets and generated association
  syntax remain gated until solver semantics exist.
- diagnostics JSON: `emit_json_diagnostics_command_outputs_machine_readable_errors`,
  `emit_json_diagnostics_includes_structured_return_keyword_fix`, and
  `emit_json_diagnostics_includes_structured_missing_bool_match_arm_fix` pin
  stable machine-readable diagnostics with suggested_fixes and feature_gate data.
- Typed/HIR/MIR JSON: `emit_json_ast_marks_semantically_unchecked_sources_that_typed_json_rejects`
  and `emit_json_typed_command_outputs_checked_program` guard checked program
  output, with HIR/MIR schema golden tests covering monomorphized structures.
- Build graph: deterministic build.zen graph behavior is guarded by parser,
  lowering, JSON, and CLI tests. Representative anchors include
  `deterministic_build_graph_creates_one_executable_target`,
  `build_graph_rejects_undeclared_host_effects`, library/test/executable
  targets, target dependencies, declared env/file reads, unsupported package
  and link gates, and `BuildGraphExecutionContext` execution routing.
- Target YAML: target schema validation covers minimal target shape, backend
  settings, C backend flags, empty flag rejection, layout override gates, and
  unsupported backend codegen gates.
- Gated effects and intrinsics:
  `async_scheduler_intrinsics_are_rejected_as_gated_not_unknown`,
  `atomic_intrinsics_are_rejected_as_effect_gates`,
  `comptime_type_match_intrinsic_is_rejected_as_gated_not_unknown`,
  `dynamic_string_type_is_rejected_as_allocator_backed_gate`,
  `raw_memory_intrinsics_are_rejected_as_allocator_gates`, and
  `syscall_intrinsics_are_rejected_as_host_effect_gates`.
- Standard library gates: promoted stdlib modules avoid removed or gated
  syntax while gated sketches such as `stdlib/io/mux/uring.zen`,
  `stdlib/io/net/unix_socket.zen`, `stdlib/io/net/socket.zen`,
  `stdlib/io/files/file.zen`, and `stdlib/sys/process/prctl.zen` remain visible
  as future surfaces.
- repo hygiene: `production_rust_files_stay_below_cleanup_threshold`,
  `zen_source_files_stay_below_cleanup_threshold`, owned spelling enums, and
  old-node cleanup tests keep files and syntax tables from regressing. Every
  tracked Rust source file should stay below the cleanup threshold or get split.

## Current Phase

Phase 5 is in evidence-hardening and cleanup. The main generic specialization
surfaces are implemented; continue by closing proof gaps, keeping generated C
consistent, and preventing large-file/slop regressions.

## Phase 5 Acceptance Evidence

- generic enum specialization: `Option<T>`, `Result<T, E>`, nested
  `Result<Option<T>, StaticString>`, duplicate variants, and multi-file enum
  dependencies are covered by executable fixtures, typed/HIR/MIR golden tests,
  and generated-C tests.
- generic method specialization: generic, `Self`, type impl, enum, imported
  dependency, and nested result method cases are covered by executable fixtures,
  JSON golden tests, and method worklist generated-C tests.
- worklist monomorphization: recursive functions, methods, imported transitive
  dependencies, and deduped instantiations are covered by `generic_worklist*`,
  `generic_method_worklist`, `multi_file_generic_imported_*`, and generated-C
  definition-count checks.
- generated-C call/definition consistency: `compile_to_c_with_generated_call_check`
  plus `undefined_generated_c_calls` scan specialization fixtures, while
  `generic_specializations_emit_each_generated_c_definition_once` guards
  duplicate definitions.
- generic arity, inference, and bound diagnostics: E5000, E5001, E5002, and
  E6004 are pinned across unit, CLI, and JSON golden tests.
- magic-string and fixture-hardcoding risk remains an active hygiene concern:
  owned spelling enums and docs-truth repo-hygiene checks should keep replacing
  ad hoc semantic string checks when they appear.

Current non-Phase-5 gaps remain Dev UX, Agent UX, full LSP/editor workflows,
allocator-backed dynamic strings, Sync/Async lowering, raw memory semantics,
advanced comptime type matching, and broad package/link build-driver behavior.

## Next Small Slice

Recommended next slice:

1. Pick one oversized Rust file that still crosses the cleanup threshold.
2. Add or tighten a focused repo-hygiene/test guard for the boundary being
   extracted.
3. Move one coherent responsibility into a focused module.
4. Run local gates before pushing: `cargo fmt --check`, `git diff --check`,
   focused tests, `cargo clippy -- -D warnings`, `cargo test --lib`, and
   `cargo test --tests`.
5. Confirm GitHub Actions stay quiet on normal branch pushes before opening a
   ready PR.

Do not mark the broader objective complete until the completion audit confirms
all required language, docs, CI, Dev UX, Agent UX, and build-driver evidence.

## Detailed Evidence References

- `docs/V1_SPEC.md`: current language/spec surface.
- `docs/DIAGNOSTICS.md`: stable diagnostic codes and JSON expectations.
- `docs/learn_zen_in_y_minutes.md`: concise public language tour.
- `docs/COMPLETION_AUDIT.md`: objective checklist, evidence summary, and gaps.
- `tests/docs_truth`: documentation and repo-shape truth gates.
- `tests/integration`: CLI, JSON, build graph, diagnostics, and generated-C
  behavior.
- `tests/resolver_phase2.rs`: resolver Phase 2 semantic evidence.
- `tests/zen`: executable language fixtures.
- Git history: detailed per-slice implementation evidence that should not be
  copied wholesale into status Markdown.
