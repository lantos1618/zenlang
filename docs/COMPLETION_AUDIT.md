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
- Generic specialization has positive executable coverage for generic functions,
  structs, enums, methods, and recursive worklist emission. The C-source
  assertions also match generated mangled generic call sites to emitted
  definitions, including struct-returning specializations without counting
  declarations or definitions as calls.
- Generic method specialization preserves concrete `Self` receiver context in
  call-site typing and specialized method bodies for generic struct and enum
  receivers, with executable and generated-C coverage in
  `tests/zen/generic_method_self.zen`, including receiver-based inference for
  `Self`-only generic method signatures and nested
  `Box<Option<i32>>` specialization dependency ordering.
- Generic method specializations that call generic functions have executable
  and generated-C coverage in `tests/zen/generic_method_worklist.zen`, including
  call-resolution assertions for the reached generic function dependency.
- Resolver method value symbols carry complete value-signature metadata, and
  typechecker setup rejects method signature drift before method body
  collection. Function-typed method parameters and returns are included in
  that handoff coverage.
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
  Generic function and method type-argument inference conflicts now report
  direct diagnostics for the conflicting parameter and concrete types.
  Generic behavior bound failures are covered across direct function, method,
  generic-receiver method, and UFC-style function call paths.
- Typechecker setup accepts resolver `SymbolTable` through
  `check_program_with_symbols`.
- Resolver behavior impl method symbols carry complete value-signature metadata,
  and typechecker setup rejects impl-method signature drift before behavior impl
  body collection. Function-typed impl-method parameters and returns are
  included in that handoff coverage.
- Resolver behavior impl method bodies carry scoped local symbols, and
  typechecker setup rejects missing impl-method body locals before behavior impl
  body collection.
- Resolver local symbols carry mutability metadata for mutable parameters and
  local bindings.
- Typechecker setup requires resolver parameter local symbols before collecting
  function or method bodies from the AST.
- Typechecker setup requires resolver local symbols for `VarDecl` bindings found
  in function or method bodies before typed body collection.
- Typechecker setup requires resolver local symbols for pattern bindings before
  checking match arm bodies.
- Resolver pattern bindings carry scoped local symbols, and typechecker setup
  rejects missing pattern locals before checking match arm bodies.
- Typechecker setup requires resolver local symbols inside top-level expression
  declarations before typed declaration collection.
- Resolver top-level expressions carry scoped local symbols, and typechecker
  setup rejects missing top-level expression locals before typed declaration
  collection.
- Resolver closure expressions carry scoped parameter/body local symbols, and
  typechecker setup rejects missing closure locals before typed body collection.
- Typechecker setup requires resolver local symbols inside struct field default
  expressions before collecting struct metadata.
- Resolver struct field default expressions carry scoped local symbols, and
  typechecker setup rejects missing default-expression locals before struct
  metadata collection.
- Typechecker setup requires resolver parameter/local symbols inside behavior
  default method bodies before collecting behavior metadata.
- Resolver behavior default method bodies carry scoped local symbols, and
  typechecker setup rejects missing default-body locals before behavior metadata
  collection.
- Typechecker setup rejects resolver local mutability mismatches before
  collecting typed bodies from the AST.
- Typechecker setup rejects resolver local visibility/source mismatches before
  collecting typed bodies from the AST.
- Typechecker setup rejects resolver local parameter-count and return-type
  metadata before collecting typed bodies from the AST.
- Typechecker setup rejects resolver local type, field, variant, and behavior
  metadata before collecting typed bodies from the AST.
- Typechecker setup mirrors resolver scope allocation for local-symbol
  validation, so same-name locals in different scopes are checked against the
  exact resolver scope.
- Resolver records same-name locals in distinct scopes as separate scoped
  symbols, matching the typechecker setup scope mirror.
- Typechecker imports can be seeded from resolver import binding symbols.
- Typechecker setup rejects resolver import binding source mismatches before
  seeding imported module-call bindings.
- Typechecker setup rejects resolver import binding visibility mismatches before
  seeding imported module-call bindings.
- Typechecker setup rejects resolver import binding parameter-count and
  return-type metadata before seeding imported module-call bindings.
- Typechecker setup rejects resolver import binding type, field, variant, and
  behavior metadata before seeding imported module-call bindings.
- Typechecker setup rejects resolver import binding mutability metadata before
  seeding imported module-call bindings.
- Typechecker setup validates resolver import binding source and visibility
  invariants even when AST import declarations have already been stripped and
  imports are seeded from resolver symbols only, including the referenced
  resolver module symbol.
- Typechecker setup rejects resolver module symbol visibility/source mismatches
  before validating imported binding symbols.
- Typechecker setup rejects resolver module parameter-count and return-type
  metadata before validating imported binding symbols.
- Typechecker setup rejects resolver module type, field, variant, and behavior
  metadata before validating imported binding symbols.
- Typechecker setup rejects resolver module mutability metadata before
  validating imported binding symbols.
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
- Resolver rejects duplicate method names inside local behavior declarations,
  covered by
  `resolver_phase2::resolver_rejects_duplicate_behavior_method_names`.
- Resolver type symbols carry behavior impl and `.requires` association
  metadata, and typechecker setup rejects missing or extra association metadata
  before collecting behavior impls/requires from the AST. Specialized behavior
  references such as `Json<str>` are included in this resolver handoff
  validation.
- Generic behavior bounds use the same inheritance-aware behavior satisfaction
  check as `.requires`.
- Inherited generic behavior dispatch is covered by the executable fixture
  `tests/zen/behavior_inherited_generic_dispatch.zen`.
- Concrete generic behavior association syntax in `.implements` and `.requires`,
  such as `Point.implements(Json<str>)`, is parsed, typechecked with substituted
  behavior method signatures, and covered by
  `tests/zen/behavior_json_generic_association.zen`.
- Generic behavior inheritance in `.extends`, including
  `PrettyJson.extends(Json<str>)`, is parsed, recorded in resolver metadata,
  checked with substituted parent methods, and covered by local and graph-owned
  multi-file fixtures.
- Unspecialized generic behaviors in `.implements`, `.requires`, and `.extends`
  produce hard arity diagnostics instead of silently acting like nongeneric
  behaviors.
- Generic behavior bounds with concrete type arguments, including
  `T: Json<T>`, are parsed, recorded in resolver metadata, checked with
  substituted behavior type arguments, and covered by
  `tests/zen/behavior_json_generic_bound.zen`.
- UFCS dispatch through substituted generic behavior bounds is covered by
  `tests/zen/behavior_json_generic_bound_ufcs.zen` plus generated-C assertions
  that reject unresolved `T_encode` calls.
- Imported public types carry source-module behavior impl associations and impl
  methods into graph-owned generic behavior-bound dispatch, covered by
  `tests/zen/multi_file_imported_behavior_impl/main.zen`.
- Imported public types carry omitted behavior default methods into graph-owned
  generic behavior-bound dispatch, covered by
  `tests/zen/multi_file_imported_behavior_default/main.zen`.
- Imported public types preserve source-module impls whose target behavior was
  itself imported by that source module, including inherited parent bounds,
  covered by `tests/zen/multi_file_imported_impl_imported_behavior/main.zen`.
- Imported behavior inheritance follows parent behavior imports from the defining
  module, with negative coverage in
  `integration::imported_behavior_extends_imported_parent_requires_parent_methods`.
- Generic dispatch through an imported child behavior can call a method inherited
  from that behavior's imported parent, covered by
  `tests/zen/multi_file_imported_child_parent_dispatch/main.zen`.
- Entry-module `.requires` assertions over imported public types and imported
  generic behaviors are covered by
  `tests/zen/multi_file_imported_behavior_requires/main.zen`.
- CLI graph-frontend typechecker failures are reported once rather than
  duplicated from both returned errors and stored checker diagnostics, covered
  by `integration::check_command_deduplicates_typechecker_diagnostics`.
- Enum variant resolver symbols are scoped by owner enum, so different enums can
  reuse variant names while same-enum duplicates remain rejected, covered by
  `resolver_phase2::resolver_allows_same_variant_names_in_different_enums` and
  `tests/zen/duplicate_enum_variant_names.zen`.
- Resolver rejects unknown enum variant expressions for local enum types before
  typechecking, covered by
  `resolver_phase2::resolver_rejects_unknown_enum_variant_expressions`.
- Resolver rejects missing or unexpected payloads on local enum variant
  expressions before typechecking, covered by
  `resolver_phase2::resolver_rejects_missing_enum_variant_payload_expressions`
  and
  `resolver_phase2::resolver_rejects_unexpected_enum_variant_payload_expressions`.
- Resolver rejects unknown type names plus duplicate, unknown, and missing
  fields on local struct literal expressions before typechecking, covered by
  `resolver_phase2::resolver_rejects_duplicate_struct_literal_fields`,
  `resolver_phase2::resolver_rejects_unknown_struct_literal_fields`,
  `resolver_phase2::resolver_rejects_missing_struct_literal_fields`, and
  `resolver_phase2::resolver_rejects_unknown_struct_literal_types`.
- Resolver rejects duplicate field names inside local struct declarations,
  covered by `resolver_phase2::resolver_rejects_duplicate_struct_field_names`.
- Unknown method calls through generic behavior-bound receivers produce hard
  diagnostics before codegen instead of unresolved `Type_method` calls.
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
- Concrete generic behavior parent inheritance, such as
  `PrettyJson.extends(Json<str>)`, is parsed, recorded in resolver metadata,
  checked with substituted parent method signatures, and covered by
  `tests/zen/behavior_generic_parent_inheritance.zen`.
- Resolver behavior symbols carry parent behavior metadata, and typechecker
  setup rejects missing or extra resolver parent-edge metadata. Specialized
  parent references such as `Json<str>` are included in this resolver handoff
  validation.
- Behavior impl coherence rejects overlapping parent/child behavior impls for
  the same type.
- Behavior impl coherence is covered for specialized generic parent/child
  overlap and for distinct generic specializations that must remain independent.
- Inherited behavior default methods are emitted and callable in the executable
  fixture `tests/zen/behavior_inherited_default_method.zen`.
- Behavior impl methods are recorded and validated through resolver
  `Type.method` value symbols.
- Top-level method declarations require a known receiver type in resolver
  diagnostics, and typechecker setup rejects missing receiver type symbols
  before collecting method metadata.
- Typechecker setup rejects extra resolver-owned declaration symbols for values,
  types, behaviors, and enum variants before declaration metadata collection.
- Typechecker setup rejects extra resolver-owned import and module symbols when
  AST import declarations are present, while preserving the resolver-symbol
  import seeding path for stripped import declarations.
- Module-graph import setup seeds imported declaration signatures directly
  instead of re-running declaration collection over cloned imported
  declarations, with coverage for function-typed imported signatures and
  imported generic function and enum specialization. Importing a public type
  also seeds its public methods and public generic method templates on the graph
  path.
- Typechecker setup rejects extra resolver-owned local symbols after mirroring
  resolver scope allocation across function bodies, nested scopes, pattern
  bindings, closures, defaults, and top-level expressions.
- Resolver rejects `Self` type references outside method, impl-method, or
  behavior contexts before typechecking can resolve them to `Unknown`.
- Direct typechecker entrypoints enforce the same `Self` context rule for
  resolver-less unit/API paths.
- Direct typechecker entrypoints reject unknown named and generic type
  references before body checking, matching resolver-backed diagnostics.
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
- Resolver value signature metadata preserves function-typed parameters and
  returns, and typechecker setup rejects function-type signature drift before
  declaration collection.
- Resolver value symbols carry generic type-parameter counts, and typechecker
  setup rejects mismatches before collecting function or method metadata.
- Resolver value symbols carry generic type-parameter names, and typechecker
  setup rejects mismatches before collecting function or method metadata.
- Resolver value symbols carry generic type-parameter bounds, and typechecker
  setup rejects mismatches before collecting function or method metadata.
- Typechecker setup rejects resolver value source, field, variant, behavior,
  and mutability metadata before collecting function or method metadata.
- Typechecker setup rejects resolver type and behavior source, value signature,
  and mutability metadata before collecting declaration metadata.
- Typechecker setup rejects resolver struct variant metadata and resolver enum
  field metadata before collecting declaration metadata.
- Typechecker setup rejects resolver behavior field, variant, impl, and
  required-behavior metadata before collecting behavior metadata.
- Typechecker setup rejects resolver variant import, value, generic, field,
  enum-type, behavior, and mutability metadata before collecting enum variant
  metadata.
- Resolver type and behavior symbols carry generic parameter-count metadata, and
  typechecker setup rejects mismatches before collecting declaration metadata.
- Resolver type and behavior symbols carry generic parameter-name metadata, and
  typechecker setup rejects mismatches before collecting declaration metadata.
- Resolver type and behavior symbols carry generic type-parameter bounds, and
  typechecker setup rejects mismatches before collecting declaration metadata.
- Typechecker setup rejects generic behavior type-parameter bound drift,
  including bounds with type arguments such as `T: Json<T>`.
- Resolver type symbols carry visibility metadata for structs and enums, and
  typechecker setup rejects mismatches before collecting declaration metadata.
- Typechecker setup rejects resolver behavior symbol visibility mismatches
  before collecting behavior metadata.
- Resolver behavior symbols carry method signature metadata, and typechecker
  setup rejects mismatches before collecting behavior metadata.
- Resolver behavior method signature metadata preserves function-typed
  parameters and returns, and typechecker setup rejects function-type method
  signature drift before behavior metadata collection.
- Resolver behavior method signature metadata preserves generic return types on
  generic behaviors, and typechecker setup rejects generic method-signature
  handoff drift before behavior metadata collection.
- Resolver top-level method and behavior impl method value symbols preserve
  function-typed parameters and returns, and typechecker setup rejects
  function-type method handoff drift before method body collection.
- Resolver struct symbols carry field-count metadata, and typechecker setup
  rejects mismatches before collecting struct field metadata.
- Resolver struct symbols carry field-name/type metadata, and typechecker setup
  rejects mismatches before collecting struct field metadata.
- Resolver struct field metadata preserves function-typed fields, and
  typechecker setup rejects function-type field drift before struct metadata
  collection.
- Resolver/typechecker handoff coverage preserves generic type parameters in
  struct fields and enum payloads before type metadata collection.
- Resolver enum variant symbols carry payload-count metadata, and typechecker
  setup rejects mismatches before collecting enum variant metadata.
- Resolver enum type symbols carry exact variant-name metadata, and typechecker
  setup rejects mismatches before collecting enum variants.
- Resolver enum variant symbols carry owner enum metadata, and typechecker setup
  rejects mismatches before collecting enum variant metadata.
- Resolver enum variant symbols carry visibility metadata, and typechecker setup
  rejects mismatches before collecting enum variant metadata.
- Resolver enum variant symbols carry payload-type metadata, and typechecker
  setup rejects mismatches before collecting enum variant metadata.
- Resolver enum variant payload metadata preserves function-typed payloads, and
  typechecker setup rejects function-type payload drift before enum variant
  metadata collection.
- CLI `build` rejects `build.zen` explicitly until deterministic build graph
  support exists, with integration coverage for the gated Phase 4 entrypoint.
- Multi-file generic import fixtures cover imported generic enum/function
  specialization through C generation and runtime execution, including generated
  C assertions that imported mangled calls have matching concrete definitions.
- Multi-file generic behavior-bound fixtures cover imported public behavior
  declarations through module-graph resolver validation, typechecking, C
  generation, and runtime execution.
- Multi-file behavior inheritance fixtures cover imported behavior parent edges
  through module-graph typechecking, including direct and transitive negative
  missing-method diagnostics and generated-C assertions for a positive
  inherited-bound specialization.

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
