# Phase Plan

## Recovery Point
Recovered branch: `codex/phase0-1-truth-gates`.
Recovery commit: `183d140c` from 2026-05-12 08:18:35 UTC.
Treat unpushed `/tmp` work after that commit as lost; continue from checked-in
docs, tests, and commits only.

## Design Decisions To Preserve
- Sync/Async are real effects, not marker-only types.
- typed allocators are central to allocation and effect decisions.
- actors live in std first; no actor syntax is v1-stable yet.
- AST/HIR traversal is tooling/metaprogramming, not core semantics.
- type matching and behavior association are separate mechanisms.
- JSON is compiler-owned IR output.
- YAML is human-authored config/spec input.
- build.zen is deterministic comptime build graph construction.
- Project imports must resolve through the stdlib surface or names exposed by
  the project root `build.zen`; ad hoc reach-through module paths are not a
  stable namespace model.
- `StaticString` is baked program data; allocator-backed `String` is dynamic.
- `Type.implements(Behavior)` covers non-generic explicit behavior associations
  until the solver supports advanced forms.
- Dev UX and Agent UX are product requirements, not polish.

## Dev UX And Agent UX Track
MoonBit-style toolchain integration is the benchmark: compiler, build graph, package surface, language server, VS Code extension, web/editor entry point, and machine-readable outputs should feel coherent.
Required Dev UX: syntax/semantic diagnostics, go-to-definition, hover, completion, formatting, run/test code lenses, target selection, language server restart, compiler version display, local toolchain validation, and `zen lsp`.
Required Agent UX: agent-readable diagnostics with stable codes, spans, related locations, suggested_fixes, feature_gate metadata, CLI/editor-aligned JSON, Machine-readable project graph and symbol graph output, deterministic quiet commands, structured fix suggestions, retrieval-friendly docs, and quiet normal branch pushes.

## Compiler And Stdlib Boundary
Compiler-owned: parsing, typing, resolver metadata, diagnostics, checked IR/JSON, build graph output, lowering, backend emission, and primitive `@builtin` hooks. Rust may expose raw hooks such as allocation, byte memory, syscalls, and atomics, but it must not own allocator policy, async API shape, scheduler composition, collection semantics, or user-facing runtime composition.

Stdlib-owned: allocator implementations, dynamic string construction, collections, IO wrappers, sync/async runtime APIs, actors, and higher-level wrappers over compiler hooks. Dynamic `String` should be implemented in stdlib on top of allocator and memory/compiler facade hooks, not as parser/compiler-owned special syntax. Raw `@builtin` calls should stay behind `stdlib/compiler.zen`; other stdlib modules should import compiler wrappers or typed stdlib abstractions.

Stdlib anti-slop pass: audit `stdlib/` for stale LLVM-era wording, direct raw-intrinsic leakage, gated syntax claims, oversized sketch files, duplicate allocator/async APIs, and modules that present experimental surfaces as promoted. Most current stdlib files are placeholders/sketches from earlier work; do not treat them as implemented evidence until the real parser, typechecker, build path, and docs-truth gates prove them. Promote only APIs with parse/typecheck/build evidence and docs-truth coverage.

## Compressed Evidence Map
This is a capability index, not a changelog. Granular evidence belongs in tests,
golden fixtures, and git history.

- Phase 0 truth gates: README, contributor docs, stdlib, CI, release, old-spec quarantine, and docs shape are guarded by `tests/docs_truth`.
- Phase 1 frontend and C-backend baseline: syntax and C execution are covered by `docs/V1_SPEC.md`, `tests/zen`, integration tests, and generated-C checks.
- generic specialization: functions, structs, enums, methods, recursive worklists, imports, nested `Result<Option<T>, StaticString>`, and generated-C consistency are covered by executable and JSON-golden integration tests.
- resolver/typechecker replay: resolver-owned metadata, callable signatures, behavior impl passes, stale AST protection, `generic_struct_constructor_without_type_args_is_error`, and generic arity diagnostics are covered by `tests/resolver_phase2.rs`, typechecker unit tests, and integration diagnostics.
- diagnostics JSON: `emit_json_diagnostics_command_outputs_machine_readable_errors`, `emit_json_diagnostics_includes_structured_return_keyword_fix`, and `emit_json_diagnostics_includes_structured_missing_bool_match_arm_fix` pin stable diagnostics with suggested_fixes and feature_gate data.
- Typed/HIR/MIR JSON: `emit_json_ast_marks_semantically_unchecked_sources_that_typed_json_rejects`
  and `emit_json_typed_command_outputs_checked_program` guard checked output.
- build graph: deterministic build.zen graph behavior is guarded by parser, lowering, JSON, CLI tests, and `deterministic_build_graph_creates_one_executable_target`.
- Gated primitive intrinsics: diagnostics golden fixtures pin `@builtin.<name>` as a guarded compiler-owned namespace. The named raw primitive surface belongs behind the Zen stdlib compiler facade, not in a Rust-owned intrinsic catalog.
- repo hygiene: file-size tests, owned spelling enums, syntax cleanup tests, and docs-truth caps prevent large-file and status-doc regressions.

## Current Phase
Phase 5 is in evidence-hardening and cleanup. The generic specialization surfaces are implemented; continue closing proof gaps, keeping generated C consistent, and preventing large-file/slop regressions. Phase 6 (FFI) has begun in parallel — see below.

## Phase 6 — Native FFI (linking C libraries)
Goal: call C libraries from Zen, declared in the project, mirroring Zig's two
layers (`linkSystemLibrary` + `extern fn`) and skipping `@cImport`/translate-c
(embedding a C parser is out of scope).

Landed:
- **`link:` on `build.zen` Executable** (Zig `linkSystemLibrary` analog): an
  un-gated DSL field threaded `BuildTargetKind::Executable` → the cc step,
  resolving each library via pkg-config (`--cflags --libs` + an rpath to its
  libdir; bare `-l<name>` fallback). `packages:` stays gated.
- **`extern` C function declarations** (Zig `extern fn` analog):
  `@extern NAME = (params) Ret` — a bodyless callable lowered to a C prototype +
  direct call with the bare C symbol name; opaque pointers as `RawPtr<u8>`. No
  header required (the prototypes are ABI-correct on their own).
- Evidence: `~/sdl3-zen` opens an SDL3 window via native `extern` calls +
  `link: ["SDL3"]`, no env vars / inline C / header; verified building and
  running headlessly under `xvfb-run`.

Next (better-than-Zig, not yet built):
- **ABI-verified extern**: emit a C static type-compat check against the real
  header so a wrong `extern` signature fails the build instead of becoming
  runtime UB — closes the silent-mismatch footgun both Zig FFI modes have.
- **Auto string marshaling**: `extern f = (s: Str)` passes a null-terminated
  `const char*` (today via `@builtin.static_string_ptr`).
- **Version-checked link**: `link: [{ name: "SDL3", min: "3.2" }]`.
- **Effect-tracked FFI** (uniquely-Zen stretch): model `extern` calls in the
  existing host-effect system so the type system knows a call reaches into C.

Out of scope for Phase 6: a package manager (`packages:` stays gated), and
`@cImport`-style header translation.

## Feature Status Confidence
| Surface | Status | Confidence |
|---|---|---|
| Generic specialization, worklist C output, and resolver replay | implemented | high |
| Diagnostics JSON, typed/HIR/MIR JSON, and build graph evidence | implemented | high |
| Behavior declarations, explicit associations, and std facades | experimental | medium |
| FFI: `extern` C functions and `link:` system libraries | implemented | high |
| Dev UX, Agent UX, LSP/editor workflows, and package driver | planned | medium |
| Async, typed allocator runtime, raw memory, syscalls, and comptime type matching | gated | high |

## Anti-Slop Scrub Queue
Use semantic-overlap and slop-cannon reports as triage input, not automatic edits. Record credible classes here; keep generated reports, embeddings, and model caches ignored.

- fixed/guarded summary: duplicate examples, stale generated/editor artifacts, raw stdlib `@builtin` leakage, unparseable stdlib sketches, repeated spelling tables, resolver/generic-callable wrappers, C emission helpers, parser declaration-mode repetition, JSON golden duplication, generated-C assertion ladders, resolver metadata fixture repetition, and small semantic-overlap helper duplication.
- slop-cannon reports: keep generated artifacts outside git; use Qwen/L40S semantic reports and cheap HEAD hash reports as triage, not automatic edit lists.
- next code cleanup: runtime/std facade stand-ins, split intrinsic registry, operator tables, builtin type/layout metadata, lexer keyword/root-token spellings, runtime descriptor enums, lossy `Type` to `AstType` conversion, generic callable specialization, behavior-ref/key normalization, resolver task routing, resolver-local traversal helpers, resolver metadata absence profiles, generic behavior-impl template insertion, call-resolution normalization, and build-target lowering specs.
- next test cleanup: generated-C substring ladders, metadata absence matrices, docs-truth prose pinning, exact temporary-name assertions, byte-identical IR golden outputs, repetitive host-effect matrices, and private-layout hygiene checks that behave like `1 == 1`.
- next docs/stdlib cleanup: public std/build facade story, async/actor/allocator sketch quarantine, duplicate allocator/async API shapes, and compact status docs that avoid repeating the phase plan.
- manual audit queue: normalize schema-heavy JSON golden coverage, quarantine blocking async allocator sketches, centralize stdlib syscall/layout constants, and make README/Learn snippets point at executable examples where repetition is not buying clarity.

## Phase 5 Acceptance Evidence
- generic enum specialization: `Option<T>`, `Result<T, E>`, nested results, duplicate variants, multi-file dependencies, executable fixtures, typed/HIR/MIR golden tests, and generated-C tests.
- generic method specialization: generic, `Self`, type impl, enum, imported dependency, nested result cases, JSON golden tests, and method worklist generated-C checks.
- worklist monomorphization: recursive functions/methods, imported transitive dependencies, deduped instantiations, and generated-C definition-count checks.
- generated-C call/definition consistency: `compile_to_c_with_generated_call_check`, `undefined_generated_c_calls`, and duplicate-definition scans.
- generic arity, inference, and bound diagnostics: E5000, E5001, E5002, and E6004 across unit, CLI, and JSON golden tests.

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
