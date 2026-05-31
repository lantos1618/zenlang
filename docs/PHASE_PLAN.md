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
- Keyword-free surface: Zen has NO hard keywords (`from_keyword` returns `None`
  for everything; guarded by `zen_has_no_hard_keywords`). All "magic" is an
  `@`-directive (`@std`/`@builtin`/`@this`/`@export`/`@extern`) or a sigil
  (`=`, `::=`, `?`, `:`, `<>`, `.`). One way to do each thing.
- Visibility: everything is private by default; the public surface is declared
  by one `@export({ Name, Type.method })` manifest per module (methods exported
  individually by dotted name, so per-method privacy is preserved). Desugared to
  the `public` flag at parse time, so the rest of the pipeline is unchanged.
  There is no `pub` keyword.
- FFI: `@extern Name = (params) Ret` declares a C function; `link: [..]` on a
  build.zen `Executable` links the library (pkg-config-resolved).

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

- Typed allocator (runtime foundation): `stdlib/memory/allocator.zen` is a real `Allocator` behavior (`alloc`/`realloc`/`free`) with a malloc-backed `Mallocator` default over the raw-memory `@builtin`s. `Vec` is `Vec<T, Alloc: Allocator = Mallocator>` — it allocates through the allocator, not hardcoded `@builtin.raw_allocate`, and the **default type parameter** lets a holder write `Vec<T>` for the system allocator (Stack/Queue no longer name `Mallocator` at all). Proven by `stdlib_vec_allocator` (a counting allocator reports `counter_allocs=2`) and `default_type_param` (`Pair<i32>` and `Pair<i32, i64>` are the same type). Default type params are filled at every type-arg substitution site *before* name mangling, so the omitted form specializes identically. Behavior-impl seeding follows the transitive import closure, so a program using Stack/Queue needs no allocator import of its own — the `Mallocator.implements(Allocator)` impl is seeded wherever its type is reachable (proven by `stdlib_stack_no_alloc_import` / `stdlib_queue_no_alloc_import`). "Type knownness" is now resolver-only (the typechecker's duplicate `E0201` was removed as redundant).
- Allocator family + syscall-backed stdlib: beyond `Mallocator`, three real strategies are pure stdlib Zen on the raw `@builtin` hooks — `Arena` (bump), `Pool` (fixed-block O(1) free list), `Heap` (first-fit free list + split), plus a `Gpa` that *statically composes* Pool+Heap and routes by size (compile-time branch, no dynamic dispatch). The `syscall*`/`atomic_*`/`fence` intrinsics are ungated (full C lowering already existed); `stdlib/compiler.zen` is the sanctioned facade (typed `sys0..sys6`, `atomic_*`, `fence`) and other modules build on it. Promoted with runtime fixtures: `mmap`/`getrandom`/`eventfd`/`process`/`sched`/`time`/`uname`/`file` (syscall-backed), `atomic`/`once` (atomics), `env` (libc FFI), `prng`/`slice`/`buffer`/`propagate`/`iterator` (pure Zen). Still gated: `async_enqueue`/`async_yield`/`type_match`.
- Async milestone 1 (surface + straight-line lowering): `@async` marks a function literal (mirrors `@extern`), `@await e` is a prefix `@`-directive; typing introduces `Type::Future(T)` with diagnostics E3080 (await outside async) / E3081 (await of non-future). The C backend lowers a straight-line async fn to a resumable frame + poll state machine driven by `block_on` — single and multiple sequential awaits (a local threaded across suspends) compile, link, and run (`async_await_ready`: 42, 23). Awaits nested in sub-expressions/branches/loops and generic async remain gated via `async_is_lowerable` (E3082); the await-handle-spill for real Pending re-poll is the documented milestone-2 next step (`docs/ASYNC_PLAN.md`).
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

Landed (L3 — better than Zig):
- **Opaque `@extern` C types**: `@extern SDL_Window` (no `=`) declares an opaque
  type, forward-declared `typedef struct N N;` and used behind pointers
  (`RawPtr<SDL_Window>` → `SDL_Window*`). Lets FFI signatures name the real C
  types, so `headers:` ABI-verify covers pointer APIs, not just primitives.
- **ABI-verified `@extern`**: `headers: ["SDL3/SDL.h"]` on the Executable
  `#include`s the real header; since codegen emits a prototype per `@extern`,
  a signature mismatch becomes a C *"conflicting types"* error at build time
  instead of runtime UB — beating both Zig FFI modes, with no static-assert
  machinery. (Caveat: a Zen global must not collide with a header *macro*.)
- **Auto string marshaling**: a `StaticString` `@extern` param lowers to
  `const char*` and call sites marshal via `zen_str.ptr` — no
  `@builtin.static_string_ptr`.
- **Version-checked `link:`**: `link: ["sdl3 >= 3.2"]` (or
  `[Lib { name: "sdl3", min: "3.2" }]`) is checked via
  `pkg-config --atleast-version` up front (`needs version >= X, found Y`).
- Evidence: `~/sdl3-zen` declares SDL_Window/SDL_Renderer opaque, uses
  `headers: ["SDL3/SDL.h"]` (ABI-verified against the real header) + a marshaled
  `StaticString` title + version-checked `link:`; builds and runs under xvfb.

Next (stretch):
- **Effect-tracked FFI** (uniquely-Zen): model `@extern` calls in the existing
  host-effect system so the type system knows a call reaches into C.

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
