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
  declarations or definitions as calls. The generated-C specialization test now
  also scans every covered generated call with an underscore-style mangled name
  and fails if it has no emitted definition. Every executable integration
  fixture also runs that generated-call scan before compiling C. Worklist dedup
  coverage now counts generated C function definitions directly, so prototypes
  no longer stand in for "emitted once" evidence.
- Resolver method symbols carry full value-signature metadata, including
  generic type-parameter names and bounds, and typechecker setup validates
  method signature handoff drift before method bodies are checked.
  Function-typed method parameters and returns are included in that resolver
  handoff coverage.
- Resolver-backed generic type-reference validation now reads collected
  resolver-restored function and method signatures, so stale AST-only parameter
  or return annotations cannot produce false unknown-type diagnostics.
- Resolver-backed top-level method collection also restores method names from
  resolver value symbols by declaration span when AST-only method names are
  stale, so collected `Type.method` signatures use resolver-owned names.
- Resolver-backed non-behavior `Type.impl` method collection now uses the same
  resolver-owned declaration-span handoff, so stale AST-only impl method names
  cannot leave stale `Type.missing` method entries during setup. Generic
  `Type.impl` method templates are covered by the same resolver-restored key,
  parameter, and return metadata path.
- Resolver-backed generic type-reference validation now also derives scoped
  generic type parameters and struct, enum, behavior, and impl-method
  declaration type references from collected resolver-restored metadata, so
  stale AST-only generic parameter names cannot produce false unknown-type
  diagnostics.
- Resolver-backed generic bound validation now defers AST-only type-parameter
  constraint checks until resolver metadata has been restored for functions,
  structs, enums, behaviors, and impl methods, so stale AST-only behavior
  constraints cannot produce false generic-bound diagnostics.
- Resolver-backed typechecker collection now updates generic function and
  generic method templates with validated resolver type-parameter,
  bound-ref, parameter-type, and return metadata, so monomorphization templates
  no longer keep stale AST-only generic names, bounds, or function-type
  signatures after resolver validation.
- Resolver-backed generic template collection now also derives return-type
  presence from validated resolver metadata, so stale AST-only missing return
  annotations cannot erase resolver-owned generic function or method returns
  before monomorphization.
- Resolver-backed generic template collection now rebuilds template parameters
  from validated resolver parameter names and types, so stale AST-only
  parameter counts cannot leave monomorphization templates with missing or
  extra parameters.
  The rebuild preserves AST-only parameter mutability by positional fallback
  when resolver-restored parameter names differ from stale AST names.
- Resolver-backed behavior method collection now rebuilds behavior parameters
  from resolver-owned parameter names and types, so stale AST-only missing or
  extra parameters cannot distort impl conformance checks.
- Resolver-backed behavior method collection now also walks resolver-owned
  behavior method metadata in resolver order, so stale AST-only missing behavior
  methods cannot drop required methods from impl conformance checks.
- Typechecker resolver validation now derives behavior method display
  signatures and typed method metadata from one shared expectation pass, so the
  two resolver handoff checks cannot drift while scanning the same behavior
  method list.
- Typechecker resolver validation now derives value parameter count, names,
  display types, and typed parameter metadata from one shared expectation pass,
  keeping those resolver value-signature handoff checks aligned.
- Typechecker resolver validation now derives type-parameter counts, names,
  display bounds, and typed bound refs from one shared expectation pass for
  value and type-like symbols, reducing duplicate resolver metadata handoff
  construction.
- Typechecker resolver validation now derives struct field count, display
  metadata, and typed field metadata from one shared expectation pass, keeping
  those resolver field handoff checks aligned.
- Typechecker resolver validation now derives enum variant payload count,
  display type, and typed payload metadata from one shared expectation pass,
  keeping those resolver variant handoff checks aligned.
- Resolver-backed struct and enum collection now also uses typed resolver
  generic bound refs, so generic type templates no longer retain stale AST-only
  behavior bounds after resolver validation.
- Generic method specialization preserves concrete `Self` receiver context in
  both call-site typing and specialized method bodies for generic struct and
  enum receivers, covered by `tests/zen/generic_method_self.zen`. `Self`-only
  generic methods also infer their type arguments from the concrete receiver
  type. Nested generic receiver inference preserves inner generic type
  structure and emits inner specializations before containing generic structs.
  Generic method specializations that call generic functions now have worklist
  coverage so reachable generic function dependencies are emitted once,
  including public generic methods imported from another module whose bodies
  call private source-module generic functions or methods.
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
  errors, including receiver-derived generic method type arguments that
  conflict with later call arguments.
  Generic inference now also walks function, array, and raw-pointer parameter
  shapes, so nested type parameters inside compound arguments can produce
  direct conflict diagnostics.
  Resolver now rejects duplicate generic type-parameter names across value,
  type, and behavior declarations, covered by
  `resolver_phase2::resolver_rejects_duplicate_type_parameter_names`.
  Generic behavior bound failures are covered across direct function, method,
  generic-receiver method, and UFC-style function call paths.
- Explicit behavior declarations, impl conformance, default methods, generic
  behavior bounds, and explicit impl method emission have parser, typechecker,
  and executable coverage.
- Resolver records behavior default-method body locals, and typechecker setup
  requires those local symbols before behavior metadata collection.
- Omitted behavior default methods now refresh their method-table signatures
  from validated resolver behavior method metadata, including function-typed
  default method parameters and returns.
- Resolver-backed declaration collection now defers impl/requires semantic
  checks until after resolver value and behavior metadata has been restored.
- Behavior impl methods are resolver-owned value symbols with parameter,
  return, generic-name, and generic-bound metadata, and typechecker setup
  validates impl-method signature handoff drift before checking impl bodies.
  Function-typed impl-method parameters and returns are included in that
  resolver handoff coverage.
- Behavior impl conformance checks now read the collected `Type.method`
  signature, including resolver-restored impl-method metadata, so stale
  AST-only method signatures cannot produce false impl diagnostics.
- Resolver-backed behavior impl conformance now also restores impl method names
  from resolver-owned value symbols when AST-only impl method names are stale,
  without masking real extra impl methods that lack resolver-owned required
  method symbols.
- Resolver records impl-method body locals in their nested scopes, and
  typechecker setup requires those local symbols before checking impl bodies.
- Generic behavior bounds share the behavior inheritance solver, so an impl of a
  child behavior can satisfy a parent behavior bound.
- Resolver symbols now carry typed behavior association metadata for
  `.extends`, `.implements`, and `.requires`, and resolver-backed typechecker
  collection uses that structured metadata for inherited parents and behavior
  impls instead of relying only on AST association reconstruction.
  Typechecker setup also validates those structured behavior refs for generic
  parent, impl, and required-behavior associations before declaration
  collection, so display-name metadata cannot hide typed association drift.
- Resolver-backed behavior inheritance checks now validate restored resolver
  parent refs before cycle and method-coherence checks, so stale AST-only parent
  names or type arguments cannot leak false extends diagnostics.
- Resolver-backed `.requires` conformance checks now read validated resolver
  required-behavior refs, so stale AST-only required behavior type arguments
  cannot produce false missing-impl diagnostics.
- Resolver-backed `.implements` conformance checks now read validated resolver
  behavior impl refs before method conformance, so stale AST-only impl behavior
  type arguments cannot produce false method signature diagnostics.
- Resolver-backed `.implements` and `.requires` conformance now also falls back
  to declaration-order resolver refs when AST-only behavior names are stale, so
  validated resolver behavior associations cannot be shadowed by stale AST
  names during semantic checks.
- Inherited generic behavior dispatch has executable coverage through
  `tests/zen/behavior_inherited_generic_dispatch.zen`.
- Concrete generic behavior association syntax in `.implements` and `.requires`,
  such as `Point.implements(Json<str>)`, has parser, typechecker, and executable
  coverage through `tests/zen/behavior_json_generic_association.zen`.
- Generic behavior inheritance in `.extends`, including
  `PrettyJson.extends(Json<str>)`, now has parser, resolver metadata,
  typechecker substitution, local executable coverage, and graph-owned
  multi-file import coverage.
- Unspecialized generic behaviors in `.implements`, `.requires`, and `.extends`
  now produce hard arity diagnostics instead of silently acting like
  nongeneric behaviors.
- Generic behavior bounds with concrete type arguments, including
  `T: Json<T>`, now have parser, resolver metadata, typechecker substitution,
  and executable coverage through `tests/zen/behavior_json_generic_bound.zen`.
- Generic behavior declarations also enforce their own type-parameter bounds
  when concrete behavior type arguments are instantiated, with positive and
  negative typechecker coverage for `Serializable<T: Json<T>>`.
- Generic behavior inheritance accepts parent type arguments that reference the
  child behavior's own type parameters, deferring those bound checks until a
  concrete behavior specialization is instantiated.
- UFCS dispatch through a substituted generic behavior bound is covered by
  `tests/zen/behavior_json_generic_bound_ufcs.zen` and generated-C checks that
  reject unresolved `T_encode` calls.
- Imported public types now carry source-module behavior impl associations and
  impl methods into graph-owned generic behavior-bound dispatch, covered by
  `tests/zen/multi_file_imported_behavior_impl/main.zen`. Private
  source-module behavior impls are not exported as direct methods on imported
  public types.
- Imported public types also carry omitted behavior default methods into
  graph-owned generic behavior-bound dispatch, covered by
  `tests/zen/multi_file_imported_behavior_default/main.zen`.
- Imported public types now preserve source-module impls whose target behavior
  was itself imported by that source module, including inherited parent bounds,
  covered by `tests/zen/multi_file_imported_impl_imported_behavior/main.zen`.
- Imported behavior inheritance now follows parent behavior imports from the
  defining module, with negative coverage in
  `integration::imported_behavior_extends_imported_parent_requires_parent_methods`.
- Generic dispatch through an imported child behavior can call a method inherited
  from that behavior's imported parent, covered by
  `tests/zen/multi_file_imported_child_parent_dispatch/main.zen`.
- Entry-module `.requires` assertions over imported public types and imported
  generic behaviors are covered by
  `tests/zen/multi_file_imported_behavior_requires/main.zen`.
- Imported public generic functions can use behavior bounds whose behavior was
  imported by the source module, covered by
  `tests/zen/multi_file_imported_function_imported_behavior_bound/main.zen`.
- Imported public function signatures now seed public source-module parameter
  and return-type dependencies plus their behavior impl associations even when
  the entry module imports only the functions, covered by
  `tests/zen/multi_file_imported_function_param_type_dependency/main.zen`,
  `tests/zen/multi_file_imported_function_return_type_dependency/main.zen`,
  `tests/zen/multi_file_imported_function_imported_return_type_behavior/main.zen`,
  while `integration::imported_function_signature_type_dependencies_are_not_directly_visible`
  proves those signature dependencies are not directly constructible without an
  entry-module import.
- Imported public generic functions also carry source-module imported generic
  enum return dependencies through graph-owned imports, covered by
  `tests/zen/multi_file_imported_generic_function_return_enum_dependency/main.zen`.
- CLI graph-frontend typechecker failures are reported once rather than
  duplicated from both returned errors and stored checker diagnostics, covered
  by `integration::check_command_deduplicates_typechecker_diagnostics`.
- Enum variant resolver symbols are scoped by owner enum, so different enums can
  reuse variant names while same-enum duplicates remain rejected, covered by
  `resolver_phase2::resolver_allows_same_variant_names_in_different_enums` and
  `tests/zen/duplicate_enum_variant_names.zen`.
- Resolver now rejects unknown enum variant expressions for local enum types
  before typechecking, covered by
  `resolver_phase2::resolver_rejects_unknown_enum_variant_expressions`.
- Resolver now rejects missing or unexpected payloads on local enum variant
  expressions before typechecking, covered by
  `resolver_phase2::resolver_rejects_missing_enum_variant_payload_expressions`
  and
  `resolver_phase2::resolver_rejects_unexpected_enum_variant_payload_expressions`.
- Resolver now rejects unknown type names plus duplicate, unknown, and missing
  fields on local struct literal expressions before typechecking, covered by
  `resolver_phase2::resolver_rejects_duplicate_struct_literal_fields`,
  `resolver_phase2::resolver_rejects_unknown_struct_literal_fields`,
  `resolver_phase2::resolver_rejects_missing_struct_literal_fields`, and
  `resolver_phase2::resolver_rejects_unknown_struct_literal_types`.
- Resolver now rejects duplicate field names inside local struct declarations,
  covered by `resolver_phase2::resolver_rejects_duplicate_struct_field_names`.
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
- Resolver now rejects duplicate method names inside local behavior
  declarations, covered by
  `resolver_phase2::resolver_rejects_duplicate_behavior_method_names`.
- Resolver now rejects duplicate parameter names in behavior method signatures
  before typechecker metadata collection, covered by
  `resolver_phase2::resolver_rejects_duplicate_signature_parameter_names`.
- Resolver type symbols now carry behavior impl and `.requires` association
  metadata, and typechecker setup rejects missing or extra association metadata
  before collecting behavior impls/requires from the AST.
  Specialized behavior references such as `Json<str>` are included in this
  resolver handoff validation.
  Resolver now rejects duplicate local `.implements` edges before recording
  duplicate metadata, covered by
  `resolver_phase2::resolver_rejects_duplicate_behavior_impl_edges`.
  Resolver now rejects duplicate local `.requires` edges before recording
  duplicate metadata, covered by
  `resolver_phase2::resolver_rejects_duplicate_behavior_required_edges`.
  Typechecker setup also rejects extra resolver-owned typed behavior parent,
  impl, and requires refs even when display-name metadata still matches the AST.
- Behavior inheritance `.extends` now has parser, resolver, and typechecker
  coverage for inherited required methods, parent behavior satisfaction, and
  coherence diagnostics for duplicate edges, cycles, and conflicting inherited
  method signatures.
  Resolver now rejects duplicate local parent edges before recording duplicate
  metadata, covered by
  `resolver_phase2::resolver_rejects_duplicate_behavior_parent_edges`.
- Concrete generic behavior parent inheritance, such as
  `PrettyJson.extends(Json<str>)`, now has parser, resolver metadata,
  typechecker substitution, and executable coverage through
  `tests/zen/behavior_generic_parent_inheritance.zen`.
- Resolver behavior symbols now carry parent behavior metadata, and typechecker
  setup rejects missing or extra resolver parent-edge metadata. Specialized
  parent references such as `Json<str>` are included in this resolver handoff
  validation.
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
- Typechecker setup now rejects resolver import binding display and typed type,
  field, variant, and behavior metadata before seeding imported module-call
  bindings.
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
- Typechecker setup now rejects resolver module display and typed type, field,
  variant, and behavior metadata before validating imported binding symbols.
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
- Typechecker setup now rejects resolver local display and typed type, field,
  variant, and behavior metadata before collecting typed bodies from the AST.
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
- Module-graph import setup now seeds imported declaration signatures directly
  instead of re-running declaration collection over cloned imported
  declarations. Function-typed imported signatures and imported generic
  function and enum specializations are covered, and importing a public type
  also seeds its public methods and public generic method templates on the graph
  path without seeding private imported methods. Generated-C assertions now
  cover imported public generic top-level methods through
  `tests/zen/multi_file_type_method/main.zen`, and imported public generic
  method templates can specialize private source-module generic function and
  method helper calls, including helpers imported by the source module, without
  exposing those helpers to entry modules, covered by
  `tests/zen/multi_file_type_method_worklist/main.zen`,
  `tests/zen/multi_file_type_method_method_dependency/main.zen`, and
  `tests/zen/multi_file_type_method_imported_dependency/main.zen`. Imported
  public generic non-behavior `Type.impl` methods also carry source-module
  imported generic type and method dependencies only during specialization,
  covered by
  `tests/zen/multi_file_type_impl_imported_type_dependency/main.zen`.
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
- Multi-file generic import fixtures now compile and run through graph-owned
  imports, and generated-C assertions prove imported generic enum/function
  specializations resolve to concrete definitions instead of unspecialized calls.
  Imported public generic function templates also carry source-module imported
  generic type and method dependencies only during specialization, covered by
  `tests/zen/multi_file_generic_imported_type_dependency/main.zen`. Transitive
  imported generic helper template dependencies are covered by
  `tests/zen/multi_file_generic_imported_transitive_dependency/main.zen`.
- Multi-file generic behavior-bound fixtures now compile and run through
  graph-owned imports, proving imported public behaviors can satisfy
  `T: Json<T>` bounds and dispatch to concrete generated C functions.
- Multi-file behavior inheritance fixtures now compile and run through
  graph-owned imports, proving imported child behavior impls carry inherited
  parent requirements, including transitive parent chains, and can satisfy
  imported parent behavior bounds.
- Resolver and typechecker symbol validation now cover behavior impl methods as
  `Type.method` value symbols, closing another declaration handoff gap.
- Resolver rejects top-level methods whose receiver type is unknown, and
  typechecker setup requires the resolver-owned receiver type symbol before
  collecting method metadata from the AST.
- Imported public generic top-level methods compile through the module graph and
  emit concrete generated-C call/definition pairs, covered by
  `tests/zen/multi_file_type_method/main.zen`.
- Non-behavior `Type.impl = { ... }` blocks now parse, resolve as
  `Type.method` value symbols, typecheck, and emit concrete method functions,
  including generic impl methods. Covered by `parser::tests::parse_impl_block`,
  `resolver_phase2::resolver_accepts_non_behavior_impl_blocks_as_method_symbols`,
  `tests/zen/type_impl_methods.zen`, `tests/zen/multi_file_type_impl/main.zen`,
  and generated-C assertions in
  `integration::generic_specializations_do_not_emit_unspecialized_c_symbols`;
  `integration::imported_private_type_impl_methods_are_not_visible` covers the
  graph-owned import privacy boundary. Duplicate non-behavior impl method names
  and collisions with top-level `Type.method` declarations are rejected by
  resolver coverage.
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
- Resolver value signature metadata preserves function-typed parameters and
  returns, and typechecker setup rejects function-type signature handoff drift
  before collecting declarations from the AST.
- The resolver-backed typechecker path now collects function and method
  signatures from validated resolver value symbols, including typed
  function-signature metadata, instead of rebuilding those signatures only from
  AST declarations after validation.
- Resolver value symbols now carry generic type-parameter counts, and
  typechecker setup rejects mismatches before collecting function or method
  metadata from the AST.
- Resolver value symbols now carry generic type-parameter names, and
  typechecker setup rejects mismatches before collecting function or method
  metadata from the AST.
- Resolver value symbols now carry display and typed-ref generic
  type-parameter bounds, and typechecker setup rejects mismatches before
  collecting function or method metadata from the AST.
- Typechecker setup now rejects resolver value source, display and typed field,
  variant, behavior, and mutability metadata before collecting function or
  method metadata from the AST.
- Typechecker setup now rejects resolver type and behavior source, display and
  typed value-signature metadata, and mutability metadata before collecting
  declaration metadata from the AST.
- Typechecker setup now rejects resolver struct display and typed variant
  metadata and resolver enum display and typed field metadata before collecting
  declaration metadata from the AST.
- Typechecker setup now rejects resolver behavior display and typed field,
  variant, impl, and required-behavior metadata before collecting behavior
  metadata from the AST.
- Typechecker setup now rejects resolver variant import, display and typed
  value, generic, field, enum-type, behavior, and mutability metadata before
  collecting enum variant metadata from the AST.
- Resolver type and behavior symbols now carry generic type-parameter counts,
  and typechecker setup rejects mismatches before collecting struct, enum, or
  behavior metadata from the AST.
- Resolver type and behavior symbols now carry generic type-parameter names,
  and typechecker setup rejects mismatches before collecting struct, enum, or
  behavior metadata from the AST.
- Resolver type and behavior symbols now carry generic type-parameter bounds,
  and typechecker setup rejects mismatches before collecting struct, enum, or
  behavior metadata from the AST.
- Typechecker setup rejects generic behavior type-parameter bound drift,
  including bounds with type arguments such as `T: Json<T>`.
- Resolver type symbols now carry visibility metadata for structs and enums, and
  typechecker setup rejects mismatches before collecting declaration metadata
  from the AST.
- Typechecker setup now rejects resolver behavior symbol visibility mismatches
  before collecting behavior metadata from the AST.
- Resolver behavior symbols now carry method signature metadata, and typechecker
  setup rejects mismatches before collecting behavior metadata from the AST.
- Resolver behavior method signature metadata preserves function-typed
  parameters and returns, and typechecker setup rejects function-type method
  signature handoff drift before collecting behavior metadata from the AST.
- Typechecker setup also validates resolver typed behavior method metadata, so
  stale `behavior_method_types` cannot survive behind matching display
  signatures before resolver-backed behavior collection.
- The resolver-backed typechecker path now collects behavior method signatures
  from validated resolver behavior symbols, including typed function-method
  metadata, instead of rebuilding behavior method signatures only from AST
  declarations after validation.
- Resolver-backed behavior method collection now also restores method names
  from validated resolver metadata, so stale AST-only behavior method names
  cannot shadow resolver-owned signatures during impl conformance.
- Resolver-backed behavior method collection now derives return-type presence
  from validated resolver metadata, so stale AST-only missing return
  annotations cannot erase resolver-owned behavior method returns.
- Resolver-backed behavior default synthesis now runs after resolver behavior
  and impl-method metadata restoration, and restored impl method names count as
  explicit overrides so defaults cannot overwrite explicit impl signatures.
- Behavior default synthesis now uses resolver-owned behavior impl refs when
  AST-only impl behavior names or type arguments are stale, so omitted defaults
  come from the validated behavior association.
- Resolver behavior method signature metadata preserves generic return types on
  generic behaviors, and typechecker setup rejects generic method-signature
  handoff drift before behavior metadata collection.
- Resolver behavior method signature metadata also preserves function-typed
  parameters and returns over generic type parameters, and typechecker setup
  rejects that generic function-type method handoff drift before behavior
  metadata collection.
- Resolver top-level method and behavior impl method value symbols preserve
  function-typed parameters and returns, and typechecker setup rejects
  function-type method handoff drift before collecting method bodies.
- Typechecker setup validates typed resolver value-signature metadata, so stale
  `parameter_types` or `return_type` cannot survive behind matching display
  signature strings before resolver-backed value collection.
- Resolver struct symbols now carry field-count metadata, and typechecker setup
  rejects mismatches before collecting struct field metadata from the AST.
- Resolver struct symbols now carry field-name/type metadata, and typechecker
  setup rejects mismatches before collecting struct field metadata from the AST.
- Resolver struct field metadata preserves function-typed fields, and
  typechecker setup rejects function-type field handoff drift before collecting
  struct metadata from the AST.
- Typechecker setup also validates typed resolver struct field metadata, so
  stale `field_types` cannot survive behind matching field display strings
  before resolver-backed struct collection.
- The resolver-backed typechecker path now collects struct field metadata from
  validated resolver type symbols, including typed function-field metadata,
  instead of rebuilding struct fields only from AST declarations after
  validation.
- Resolver/typechecker handoff coverage preserves generic type parameters in
  struct fields and enum payloads before type metadata collection.
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
- Resolver enum variant payload metadata preserves function-typed payloads, and
  typechecker setup rejects function-type payload handoff drift before
  collecting enum variant metadata from the AST.
- Typechecker setup also validates typed resolver enum payload metadata, so
  stale `variant_payload_type` cannot survive behind matching payload display
  strings before resolver-backed enum collection.
- The resolver-backed typechecker path now collects enum variant payload
  metadata from validated resolver variant symbols, including typed
  function-payload metadata, instead of rebuilding enum payloads only from AST
  declarations after validation.
- Resolver enum variant payload metadata also preserves function-typed payloads
  over generic type parameters, and typechecker setup rejects that generic
  handoff drift before enum variant metadata collection.
- Module graph typechecking now seeds graph imports and then uses the same
  resolver-backed declaration collection as single-file resolver/typechecker
  integration, so graph-owned modules no longer fall back to plain AST
  declaration collection after resolver validation.
- The CLI `check`, `emit`, and `build` paths now reject `build.zen` explicitly
  until deterministic build graph support exists, with integration coverage for
  the gated Phase 4 entrypoints.
- Resolver behavior-association validation now shares the parent/impl/requires
  name/ref diagnostic plumbing while keeping each path's explicit labels and
  error codes.
- Resolver behavior-association collection now shares resolver behavior-ref
  symbol lookup across parent, impl, and requires handoff paths.
- Resolver behavior-association validation now builds expected display names
  and typed refs in the same AST pass for impl/requires and parent edges.
- Resolver behavior-association expectation storage now uses a shared edge
  container for impl, requires, and parent validation inputs.
- Resolver value-signature expectation building now derives parameter names,
  display types, and typed metadata in one parameter pass.

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
