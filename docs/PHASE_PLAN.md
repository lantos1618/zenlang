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
  structs, enums, methods, and recursive worklist emission. The C-source
  assertions also match generated mangled generic call sites to emitted
  definitions, including struct-returning specializations without counting
  declarations or definitions as calls.
- Resolver method symbols carry full value-signature metadata, including
  generic type-parameter names and bounds, and typechecker setup validates
  method signature handoff drift before method bodies are checked.
- Generic method specialization preserves concrete `Self` receiver context in
  both call-site typing and specialized method bodies for generic struct and
  enum receivers, covered by `tests/zen/generic_method_self.zen`. `Self`-only
  generic methods also infer their type arguments from the concrete receiver
  type. Nested generic receiver inference preserves inner generic type
  structure and emits inner specializations before containing generic structs.
  Generic method specializations that call generic functions now have worklist
  coverage so reachable generic function dependencies are emitted once.
- Generic struct and enum type-argument arity diagnostics cover both expression
  instantiation and type annotation positions, including bare generic
  annotation names with missing type arguments and local variable annotations.
  Nested generic type arguments are checked recursively in annotations,
  instantiation positions, function type signatures, and pointer/slice/array
  container types. Local generic annotations also enforce declared generic
  behavior bounds.
  Explicit generic call type arguments, closure signatures, and cast targets
  are included in the same annotation validation path.
  Generic method explicit type arguments also reject bare generic type
  annotations with missing type arguments.
  Generic function and method type-argument inference conflicts now produce
  direct diagnostics instead of relying only on substituted argument mismatch
  errors.
  Generic behavior bound failures are covered across direct function, method,
  generic-receiver method, and UFC-style function call paths.
- Explicit behavior declarations, impl conformance, default methods, generic
  behavior bounds, and explicit impl method emission have parser, typechecker,
  and executable coverage.
- Resolver records behavior default-method body locals, and typechecker setup
  requires those local symbols before behavior metadata collection.
- Behavior impl methods are resolver-owned value symbols with parameter,
  return, generic-name, and generic-bound metadata, and typechecker setup
  validates impl-method signature handoff drift before checking impl bodies.
- Resolver records impl-method body locals in their nested scopes, and
  typechecker setup requires those local symbols before checking impl bodies.
- Generic behavior bounds share the behavior inheritance solver, so an impl of a
  child behavior can satisfy a parent behavior bound.
- Inherited generic behavior dispatch has executable coverage through
  `tests/zen/behavior_inherited_generic_dispatch.zen`.
- Concrete generic behavior association syntax in `.implements` and `.requires`,
  such as `Point.implements(Json<str>)`, has parser, typechecker, and executable
  coverage through `tests/zen/behavior_json_generic_association.zen`.
- Generic behavior inheritance in `.extends` is still explicitly gated with
  parser diagnostics.
- Unspecialized generic behaviors in `.implements`, `.requires`, and `.extends`
  now produce hard arity diagnostics instead of silently acting like
  nongeneric behaviors.
- Generic behavior bounds with concrete type arguments, including
  `T: Json<T>`, now have parser, resolver metadata, typechecker substitution,
  and executable coverage through `tests/zen/behavior_json_generic_bound.zen`.
- UFCS dispatch through a substituted generic behavior bound is covered by
  `tests/zen/behavior_json_generic_bound_ufcs.zen` and generated-C checks that
  reject unresolved `T_encode` calls.
- Unknown method calls through generic behavior-bound receivers now produce hard
  diagnostics before codegen instead of unresolved `Type_method` calls.
- Unspecialized generic behavior bounds such as `T: Json`, where `Json` declares
  type parameters, now produce hard arity diagnostics instead of silently acting
  like nongeneric behavior bounds.
- Generic behavior declaration bounds are validated after all behavior names are
  collected, so bounded behavior declarations do not depend on source order.
- Unspecialized generic type targets in `.implements` and `.requires`, such as
  `Box.implements(Json)` when `Box` declares type parameters, now produce hard
  arity diagnostics instead of resolver handoff mismatch diagnostics.
- Compile-time `.requires` behavior assertions now have parser, resolver, and
  typechecker coverage for satisfied and missing behavior implementations.
- Resolver type symbols now carry behavior impl and `.requires` association
  metadata, and typechecker setup rejects missing or extra association metadata
  before collecting behavior impls/requires from the AST.
- Behavior inheritance `.extends` now has parser, resolver, and typechecker
  coverage for inherited required methods, parent behavior satisfaction, and
  coherence diagnostics for duplicate edges, cycles, and conflicting inherited
  method signatures.
- Concrete generic behavior parent inheritance, such as
  `PrettyJson.extends(Json<str>)`, now has parser, resolver metadata,
  typechecker substitution, and executable coverage through
  `tests/zen/behavior_generic_parent_inheritance.zen`.
- Resolver behavior symbols now carry parent behavior metadata, and typechecker
  setup rejects missing or extra resolver parent-edge metadata.
- Behavior impl coherence rejects overlapping parent/child behavior impls for
  the same type.
- Behavior impl coherence is now covered for specialized generic parent/child
  overlap and for distinct generic specializations that must remain independent.
- Inherited behavior default methods have executable coverage through
  `tests/zen/behavior_inherited_default_method.zen`.
- Resolver Phase 2 has started with symbol IDs, separate namespaces, duplicate
  same-namespace diagnostics, symbol visibility metadata, and unknown type
  reference diagnostics in `tests/resolver_phase2.rs`.
- Resolver import declarations now produce explicit import binding symbols with
  source module metadata instead of relying on ad hoc imported-name collection.
- Typechecker setup now rejects resolver import binding source mismatches before
  seeding imported module-call bindings.
- Typechecker setup now rejects resolver import binding visibility mismatches
  before seeding imported module-call bindings.
- Typechecker setup now rejects resolver import binding parameter-count and
  return-type metadata before seeding imported module-call bindings.
- Typechecker setup now rejects resolver import binding type, field, variant,
  and behavior metadata before seeding imported module-call bindings.
- Typechecker setup now rejects resolver import binding mutability metadata
  before seeding imported module-call bindings.
- Typechecker setup now validates resolver import binding source and visibility
  invariants even when AST import declarations have already been stripped and
  imports are seeded from resolver symbols only, including the referenced
  resolver module symbol.
- Typechecker setup now rejects resolver module symbol visibility/source
  mismatches before validating imported binding symbols.
- Typechecker setup now rejects resolver module parameter-count and return-type
  metadata before validating imported binding symbols.
- Typechecker setup now rejects resolver module type, field, variant, and
  behavior metadata before validating imported binding symbols.
- Typechecker setup now rejects resolver module mutability metadata before
  validating imported binding symbols.
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
- Resolver records pattern binding locals, and typechecker setup requires those
  resolver-owned pattern locals before checking match arm bodies.
- Typechecker setup now requires resolver local symbols inside top-level
  expression declarations before typed declaration collection.
- Resolver records top-level expression locals, and typechecker setup requires
  those local symbols before typed declaration collection.
- Resolver records closure parameter/body locals, and typechecker setup requires
  those closure-local symbols before typed body collection.
- Typechecker setup now requires resolver local symbols inside struct field
  default expressions before collecting struct metadata.
- Resolver records struct field default-expression locals, and typechecker
  setup requires those local symbols before collecting struct metadata.
- Typechecker setup now requires resolver parameter/local symbols inside
  behavior default method bodies before collecting behavior metadata.
- Typechecker setup now rejects resolver local mutability mismatches before
  collecting typed bodies from the AST.
- Typechecker setup now rejects resolver local visibility/source mismatches
  before collecting typed bodies from the AST.
- Typechecker setup now rejects resolver local parameter-count and return-type
  metadata before collecting typed bodies from the AST.
- Typechecker setup now rejects resolver local type, field, variant, and
  behavior metadata before collecting typed bodies from the AST.
- Typechecker setup now mirrors resolver scope allocation for local-symbol
  validation, so same-name locals in different scopes are checked against the
  exact resolver scope.
- Resolver records same-name locals in distinct scopes as separate local
  symbols, matching the typechecker setup scope mirror.
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
- Resolver rejects top-level methods whose receiver type is unknown, and
  typechecker setup requires the resolver-owned receiver type symbol before
  collecting method metadata from the AST.
- Typechecker setup now rejects extra resolver-owned declaration symbols for
  values, types, behaviors, and enum variants before collecting declaration
  metadata from the AST.
- Typechecker setup now rejects extra resolver-owned import and module symbols
  when AST import declarations are present, while preserving the resolver-symbol
  import seeding path for stripped import declarations.
- Typechecker setup now rejects extra resolver-owned local symbols by mirroring
  resolver scope allocation for parameters, block locals, pattern bindings,
  closures, field defaults, behavior defaults, and top-level expressions.
- Resolver now rejects `Self` type references outside method, impl-method, or
  behavior contexts instead of letting plain functions carry unresolved `Self`
  into typechecking.
- Direct typechecker entrypoints now enforce the same `Self` context rule, so
  resolver-less unit/API paths cannot resolve invalid `Self` references to
  `Unknown`.
- Direct typechecker entrypoints now reject unknown named and generic type
  references before body checking, matching resolver-backed diagnostics.
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
- Resolver value symbols now carry generic type-parameter names, and
  typechecker setup rejects mismatches before collecting function or method
  metadata from the AST.
- Resolver value symbols now carry generic type-parameter bounds, and
  typechecker setup rejects mismatches before collecting function or method
  metadata from the AST.
- Typechecker setup now rejects resolver value source, field, variant,
  behavior, and mutability metadata before collecting function or method
  metadata from the AST.
- Typechecker setup now rejects resolver type and behavior source, value
  signature, and mutability metadata before collecting declaration metadata
  from the AST.
- Typechecker setup now rejects resolver struct variant metadata and resolver
  enum field metadata before collecting declaration metadata from the AST.
- Typechecker setup now rejects resolver behavior field, variant, impl, and
  required-behavior metadata before collecting behavior metadata from the AST.
- Typechecker setup now rejects resolver variant import, value, generic, field,
  enum-type, behavior, and mutability metadata before collecting enum variant
  metadata from the AST.
- Resolver type and behavior symbols now carry generic type-parameter counts,
  and typechecker setup rejects mismatches before collecting struct, enum, or
  behavior metadata from the AST.
- Resolver type and behavior symbols now carry generic type-parameter names,
  and typechecker setup rejects mismatches before collecting struct, enum, or
  behavior metadata from the AST.
- Resolver type and behavior symbols now carry generic type-parameter bounds,
  and typechecker setup rejects mismatches before collecting struct, enum, or
  behavior metadata from the AST.
- Resolver type symbols now carry visibility metadata for structs and enums, and
  typechecker setup rejects mismatches before collecting declaration metadata
  from the AST.
- Typechecker setup now rejects resolver behavior symbol visibility mismatches
  before collecting behavior metadata from the AST.
- Resolver behavior symbols now carry method signature metadata, and typechecker
  setup rejects mismatches before collecting behavior metadata from the AST.
- Resolver struct symbols now carry field-count metadata, and typechecker setup
  rejects mismatches before collecting struct field metadata from the AST.
- Resolver struct symbols now carry field-name/type metadata, and typechecker
  setup rejects mismatches before collecting struct field metadata from the AST.
- Resolver enum variant symbols now carry payload-count metadata, and
  typechecker setup rejects mismatches before collecting enum variant metadata
  from the AST.
- Resolver enum type symbols now carry exact variant-name metadata, and
  typechecker setup rejects mismatches before collecting enum variants from the
  AST.
- Resolver enum variant symbols now carry owner enum metadata, and typechecker
  setup rejects mismatches before collecting enum variant metadata from the AST.
- Resolver enum variant symbols now carry visibility metadata, and typechecker
  setup rejects mismatches before collecting enum variant metadata from the AST.
- Resolver enum variant symbols now carry payload-type metadata, and typechecker
  setup rejects mismatches before collecting enum variant metadata from the AST.
- The CLI `build` path now rejects `build.zen` explicitly until deterministic
  build graph support exists, with integration coverage for the gated Phase 4
  entrypoint.

## Current Phase

Continue the smallest behavior-association and resolver/typechecker hardening
slices. Phase 3 C codegen is sufficient for the current tested fixtures, but
Phase 4 build-driver work is still gated by the lack of deterministic
`build.zen` graph tests and implementation.

Do not promote gated v1 features until the relevant positive and negative tests
exist and pass through the same compiler path advertised in `docs/V1_SPEC.md`.

## Next Small Slice

Continue the next smallest behavior-association handoff or resolver/typechecker
integration slice that reduces duplicate declaration collection. Do not promote
`build.zen` until a dedicated deterministic graph test exists.
