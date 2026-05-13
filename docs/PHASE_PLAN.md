# Phase Plan

## Recovery Point

Recovered branch: `codex/phase0-1-truth-gates`

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

## Completed Evidence

- Phase 0 truth gates are implemented through README, contributor, stdlib, CI,
  release, and spec assertions in `tests/docs_truth.rs`.
- Phase 1 frontend and tested C-backend baseline are implemented for the syntax
  forms listed in `docs/V1_SPEC.md` and covered by `tests/zen`.
- Generic specialization has positive executable coverage for generic functions,
  structs, enums, methods, and recursive worklist emission.
- Explicit behavior declarations, impl conformance, default methods, generic
  behavior bounds, and explicit impl method emission have parser, typechecker,
  and executable coverage.
- Compile-time `.requires` behavior assertions now have parser, resolver, and
  typechecker coverage for satisfied and missing behavior implementations.
- Behavior inheritance `.extends` now has parser, resolver, and typechecker
  coverage for inherited required methods, parent behavior satisfaction, and
  cycle rejection.
- Resolver Phase 2 has started with symbol IDs, separate namespaces, duplicate
  same-namespace diagnostics, symbol visibility metadata, and unknown type
  reference diagnostics in `tests/resolver_phase2.rs`.
- Resolver import declarations now produce explicit import binding symbols with
  source module metadata instead of relying on ad hoc imported-name collection.
- Typechecker setup now rejects resolver import binding source mismatches before
  seeding imported module-call bindings.
- Typechecker setup now rejects resolver module symbol visibility/source
  mismatches before validating imported binding symbols.
- Resolver now walks declaration bodies enough to diagnose simple unresolved
  unqualified function calls using resolver-owned value/import symbols.
- Resolver now records scoped local symbols for parameters and local bindings,
  diagnoses duplicate same-scope local bindings, and rejects unresolved local
  identifier references.
- Resolver local symbols now carry mutability metadata for mutable parameters
  and local bindings.
- Typechecker setup now requires resolver parameter local symbols before
  collecting function or method bodies from the AST.
- Typechecker setup now requires resolver local symbols for `VarDecl` bindings
  found in function or method bodies before typed body collection.
- Typechecker setup now requires resolver local symbols for pattern bindings
  before checking match arm bodies.
- Typechecker setup now requires resolver local symbols inside top-level
  expression declarations before typed declaration collection.
- Typechecker setup now requires resolver local symbols inside struct field
  default expressions before collecting struct metadata.
- Typechecker setup now requires resolver parameter/local symbols inside
  behavior default method bodies before collecting behavior metadata.
- Typechecker setup now rejects resolver local mutability mismatches before
  collecting typed bodies from the AST.
- Typechecker setup now mirrors resolver scope allocation for local-symbol
  validation, so same-name locals in different scopes are checked against the
  exact resolver scope.
- The CLI `check` path now runs resolver diagnostics before typechecking, with
  integration coverage for resolver-owned diagnostics outside resolver-only
  tests.
- The reusable integration-test frontend helper now runs resolver diagnostics
  before typechecking. Existing fixtures also cover resolver treatment of enum
  pattern payload bindings and mutable reassignment syntax.
- Typechecker setup now accepts resolver `SymbolTable` data through
  `check_program_with_symbols`, validates declaration coverage, and both CLI and
  integration-test frontend paths pass resolver symbols into typechecking.
- Typechecker import setup now consumes resolver import binding symbols, reducing
  dependence on raw import declaration walks for module-call recognition.
- The non-merging module graph now records resolver `SymbolTable` output for
  each loaded module and rejects resolver diagnostics in dependencies before the
  graph is returned.
- Typechecker setup now has an opt-in module-graph entrypoint that validates
  the entry resolver symbols and seeds imported signatures from graph-owned
  `ImportBinding`s without merging imported declarations into the entry AST.
- The CLI `check` path now loads the module graph and reports resolver
  diagnostics from imported modules before typechecking the entry module.
- The module-graph typechecker entrypoint now typechecks imported modules before
  the entry module, and `zen check` reports imported-module type errors.
- The module-graph typechecker entrypoint now returns typed dependency
  definitions with the entry module so graph-based codegen paths can resolve
  imported calls without AST declaration merging.
- The CLI `emit` path now uses the module-graph frontend, so emitted C is based
  on graph-owned import bindings and reports imported-module type errors.
- The normal CLI `build` and direct `.zen` paths now use the module-graph
  frontend while preserving the explicit `build.zen` gate.
- The reusable integration-test frontend helper now uses the module graph, so
  fixture compilation and generated-C assertions exercise the same graph-owned
  import validation as the CLI paths.
- Resolver and typechecker symbol validation now cover behavior impl methods as
  `Type.method` value symbols, closing another declaration handoff gap.
- Typechecker resolver-symbol validation now checks enum variant symbols from
  resolver output instead of treating enum type presence as sufficient.
- Resolver value symbols now carry parameter-count metadata for functions and
  methods, and typechecker setup rejects mismatches against that resolver-owned
  signature data.
- Resolver value symbols now carry parameter-name metadata for functions and
  methods, and typechecker setup rejects mismatches before collecting function
  signatures from the AST.
- Resolver value symbols now carry visibility metadata for functions and
  methods, and typechecker setup rejects mismatches before collecting function
  signatures from the AST.
- Resolver value symbols now carry parameter-type metadata for functions and
  methods, and typechecker setup rejects mismatches before collecting function
  signatures from the AST.
- Resolver value symbols now also carry return-type metadata, and typechecker
  setup rejects return-type mismatches before collecting declarations from the
  AST.
- Resolver value symbols now carry generic type-parameter counts, and
  typechecker setup rejects mismatches before collecting function or method
  metadata from the AST.
- Resolver value symbols now carry generic type-parameter bounds, and
  typechecker setup rejects mismatches before collecting function or method
  metadata from the AST.
- Resolver type and behavior symbols now carry generic type-parameter counts,
  and typechecker setup rejects mismatches before collecting struct, enum, or
  behavior metadata from the AST.
- Resolver type and behavior symbols now carry generic type-parameter bounds,
  and typechecker setup rejects mismatches before collecting struct, enum, or
  behavior metadata from the AST.
- Resolver type symbols now carry visibility metadata for structs and enums, and
  typechecker setup rejects mismatches before collecting declaration metadata
  from the AST.
- Resolver behavior symbols now carry method signature metadata, and typechecker
  setup rejects mismatches before collecting behavior metadata from the AST.
- Resolver struct symbols now carry field-count metadata, and typechecker setup
  rejects mismatches before collecting struct field metadata from the AST.
- Resolver struct symbols now carry field-name/type metadata, and typechecker
  setup rejects mismatches before collecting struct field metadata from the AST.
- Resolver enum variant symbols now carry payload-count metadata, and
  typechecker setup rejects mismatches before collecting enum variant metadata
  from the AST.
- Resolver enum variant symbols now carry visibility metadata, and typechecker
  setup rejects mismatches before collecting enum variant metadata from the AST.
- Resolver enum variant symbols now carry payload-type metadata, and typechecker
  setup rejects mismatches before collecting enum variant metadata from the AST.
- The CLI `build` path now rejects `build.zen` explicitly until deterministic
  build graph support exists, with integration coverage for the gated Phase 4
  entrypoint.

## Current Phase

Continue Phase 2 sema/resolver hardening. Phase 3 C codegen is sufficient for the
current tested fixtures, but Phase 4 build-driver work is still gated by the lack
of deterministic `build.zen` graph tests and implementation.

Do not promote gated v1 features until the relevant positive and negative tests
exist and pass through the same compiler path advertised in `docs/V1_SPEC.md`.

## Next Small Slice

Continue Phase 2 resolver/typechecker integration by choosing the next smallest
handoff that reduces duplicate declaration collection or moves the module-graph
entrypoint deeper into advertised compiler paths. Do not promote `build.zen`
until a dedicated deterministic graph test exists.
