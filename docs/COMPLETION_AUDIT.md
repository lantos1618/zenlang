# Completion Audit

## Objective Restatement

Continue from recovered branch `codex/phase0-1-truth-gates` at recovery commit
`183d140c` from 2026-05-12 08:18:35 UTC. Treat unpushed `/tmp` work after that
commit as lost, reconstruct the durable plan from checked-in docs/tests/commits,
continue TDD-first in small tested commits, preserve the stated design decisions,
and do not assume Phase 4 is ready without evidence.

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Status |
|---|---|---|
| Verify current repo state | `git status --short --branch` showed clean branch ahead of origin during continuation; latest committed state is tracked in git history. | satisfied |
| Verify tests before continuing | Local gates have been run repeatedly: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --lib`, and `cargo test --tests`. | satisfied |
| Recover from `183d140c` | `docs/PHASE_PLAN.md` records `183d140c` as the recovery point and current history is built on top of that commit. | satisfied |
| Reconstruct missing plan | `docs/PHASE_PLAN.md` records completed evidence, current phase, and next slices. | satisfied |
| Identify completed phases from docs/tests/commits | `docs/PHASE_PLAN.md` lists Phase 0 truth gates, Phase 1 tested frontend/C backend, generic specialization, behavior work, and Phase 2 resolver/typechecker work. | satisfied |
| Continue TDD-first | New slices added failing tests before implementation, including resolver Phase 2 tests, CLI/integration resolver diagnostics, and typechecker resolver-symbol tests. | satisfied |
| Work in small tested commits | Git history after `183d140c` is split into focused implementation and plan-update commits. | satisfied |
| Preserve design decisions | See Design Decisions Preserved below and `docs/PHASE_PLAN.md`. | satisfied |
| Avoid Phase 4 assumption | `docs/PHASE_PLAN.md` states Phase 4 build-driver work is gated by missing deterministic `build.zen` graph tests and implementation. | satisfied |
| Completion audit before marking done | This `docs/COMPLETION_AUDIT.md` records objective evidence and gaps. | in progress |

## Design Decisions Preserved

- Sync/Async are real effects, not marker-only types.
- Typed allocators remain central to allocation/effect decisions.
- Actors live in std first; no actor syntax has been promoted.
- AST/HIR traversal remains tooling/metaprogramming, not core semantics.
- Type matching and behavior association remain separate.
- JSON is compiler-owned IR output.
- YAML is human-authored config/spec input.
- `build.zen` is a deterministic comptime build graph design goal, not an
  implemented stable feature.

## Verified Evidence

- `docs/PHASE_PLAN.md` is present and guarded by `tests/docs_truth.rs`.
- Resolver Phase 2 has dedicated tests in `tests/resolver_phase2.rs`.
- CLI and integration frontend paths now run resolver diagnostics before
  typechecking.
- Generic struct and enum type-argument arity diagnostics cover both expression
  instantiation and type annotation positions, including bare generic
  annotation names with missing type arguments and local variable annotations.
  Nested generic type arguments are checked recursively in annotations,
  instantiation positions, function type signatures, and pointer/slice/array
  container types. Local generic annotations also enforce declared generic
  behavior bounds.
  Explicit generic call type arguments, closure signatures, and cast targets
  are included in the same annotation validation path.
  Generic behavior bound failures are covered across direct function, method,
  generic-receiver method, and UFC-style function call paths.
- Typechecker setup accepts resolver `SymbolTable` through
  `check_program_with_symbols`.
- Resolver local symbols carry mutability metadata for mutable parameters and
  local bindings.
- Typechecker setup requires resolver parameter local symbols before collecting
  function or method bodies from the AST.
- Typechecker setup requires resolver local symbols for `VarDecl` bindings found
  in function or method bodies before typed body collection.
- Typechecker setup requires resolver local symbols for pattern bindings before
  checking match arm bodies.
- Typechecker setup requires resolver local symbols inside top-level expression
  declarations before typed declaration collection.
- Typechecker setup requires resolver local symbols inside struct field default
  expressions before collecting struct metadata.
- Typechecker setup requires resolver parameter/local symbols inside behavior
  default method bodies before collecting behavior metadata.
- Typechecker setup rejects resolver local mutability mismatches before
  collecting typed bodies from the AST.
- Typechecker setup mirrors resolver scope allocation for local-symbol
  validation, so same-name locals in different scopes are checked against the
  exact resolver scope.
- Typechecker imports can be seeded from resolver import binding symbols.
- Typechecker setup rejects resolver import binding source mismatches before
  seeding imported module-call bindings.
- Typechecker setup rejects resolver module symbol visibility/source mismatches
  before validating imported binding symbols.
- The non-merging module graph records resolver `SymbolTable` data per module
  and rejects resolver diagnostics from loaded dependency modules.
- Typechecker setup has an opt-in module-graph entrypoint that validates entry
  resolver symbols and seeds imported signatures from graph-owned
  `ImportBinding`s without merging imported declarations into the entry AST.
- The CLI `check` path now uses the module graph and reports resolver
  diagnostics from imported modules instead of checking only merged imported
  declarations.
- The module-graph typechecker entrypoint now checks imported modules before the
  entry module, so `zen check` reports imported-module type errors as well.
- Graph typechecking now returns typed dependency definitions with the entry
  module, preserving imported function bodies for future graph-based codegen
  paths without reintroducing AST declaration merging.
- The CLI `emit` path now uses the module graph, reports imported-module type
  errors, and still receives typed dependency definitions for C generation.
- The normal CLI `build` path and direct `.zen` invocation now use the module
  graph while preserving the dedicated `build.zen` rejection path.
- The reusable integration-test frontend helper now uses the module graph,
  keeping fixture compilation and generated-C checks aligned with CLI import
  validation.
- Compile-time `.requires` behavior assertions are parsed, resolved against
  known type/behavior symbols, and typechecked against explicit behavior impls.
- Generic behavior bounds use the same inheritance-aware behavior satisfaction
  check as `.requires`.
- Inherited generic behavior dispatch is covered by the executable fixture
  `tests/zen/behavior_inherited_generic_dispatch.zen`.
- Generic behavior association syntax in `.implements`, `.requires`, and
  `.extends` is explicitly gated with parser diagnostics until `Json<T>`-style
  behavior association is implemented.
- Unspecialized generic behaviors in `.implements`, `.requires`, and `.extends`
  produce hard arity diagnostics instead of silently acting like nongeneric
  behaviors.
- Generic behavior bounds such as `T: Json<T>` are explicitly gated with parser
  diagnostics until generic behavior association is implemented.
- Unspecialized generic behavior bounds such as `T: Json`, where `Json` declares
  type parameters, produce hard arity diagnostics instead of silently acting
  like nongeneric behavior bounds.
- Generic behavior declaration bounds are validated after all behavior names are
  collected, so bounded behavior declarations do not depend on source order.
- Unspecialized generic type targets in `.implements` and `.requires`, such as
  `Box.implements(Json)` when `Box` declares type parameters, produce hard
  arity diagnostics instead of resolver handoff mismatch diagnostics.
- Behavior inheritance `.extends` is parsed, resolved against known behaviors,
  and typechecked so child behavior impls must satisfy inherited parent methods
  while duplicate edges, cyclic inheritance, and conflicting inherited method
  signatures are rejected.
- Resolver behavior symbols carry parent behavior metadata, and typechecker
  setup rejects missing resolver parent-edge metadata.
- Behavior impl coherence rejects overlapping parent/child behavior impls for
  the same type.
- Inherited behavior default methods are emitted and callable in the executable
  fixture `tests/zen/behavior_inherited_default_method.zen`.
- Behavior impl methods are recorded and validated through resolver
  `Type.method` value symbols.
- Enum variants are validated through resolver `Variant` symbols during
  typechecker setup.
- Resolver value symbols carry parameter-count metadata, and typechecker setup
  rejects mismatches for functions and methods.
- Resolver value symbols carry parameter-name metadata, and typechecker setup
  rejects mismatches before collecting function or method metadata.
- Resolver value symbols carry visibility metadata, and typechecker setup
  rejects mismatches before collecting function or method metadata.
- Resolver value symbols carry parameter-type metadata, and typechecker setup
  rejects mismatches before collecting function signatures.
- Resolver value symbols carry return-type metadata, and typechecker setup
  rejects mismatches before declaration collection.
- Resolver value symbols carry generic type-parameter counts, and typechecker
  setup rejects mismatches before collecting function or method metadata.
- Resolver value symbols carry generic type-parameter bounds, and typechecker
  setup rejects mismatches before collecting function or method metadata.
- Resolver type and behavior symbols carry generic parameter-count metadata, and
  typechecker setup rejects mismatches before collecting declaration metadata.
- Resolver type and behavior symbols carry generic type-parameter bounds, and
  typechecker setup rejects mismatches before collecting declaration metadata.
- Resolver type symbols carry visibility metadata for structs and enums, and
  typechecker setup rejects mismatches before collecting declaration metadata.
- Resolver behavior symbols carry method signature metadata, and typechecker
  setup rejects mismatches before collecting behavior metadata.
- Resolver struct symbols carry field-count metadata, and typechecker setup
  rejects mismatches before collecting struct field metadata.
- Resolver struct symbols carry field-name/type metadata, and typechecker setup
  rejects mismatches before collecting struct field metadata.
- Resolver enum variant symbols carry payload-count metadata, and typechecker
  setup rejects mismatches before collecting enum variant metadata.
- Resolver enum variant symbols carry visibility metadata, and typechecker setup
  rejects mismatches before collecting enum variant metadata.
- Resolver enum variant symbols carry payload-type metadata, and typechecker
  setup rejects mismatches before collecting enum variant metadata.
- CLI `build` rejects `build.zen` explicitly until deterministic build graph
  support exists, with integration coverage for the gated Phase 4 entrypoint.

## Unresolved Gaps

- Phase 2 is not complete: resolver/typechecker integration still has duplicate
  declaration collection for richer function type metadata and residual
  resolver-owned semantic handoffs.
- Phase 4 is not complete: deterministic `build.zen` graph tests and
  implementation are still absent.
- Effect checking, typed allocator semantics, actors in std integration,
  JSON/YAML IR boundaries, and build graph execution remain gated by
  `docs/V1_SPEC.md`.

## Decision

Do not mark the objective complete. Continue Phase 2 work unless a later audit
shows the resolver/typechecker and build-gate requirements have concrete
coverage.
