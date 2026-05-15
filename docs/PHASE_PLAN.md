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
  It now also resolves stale AST declaration names through resolver symbols
  before validating collected type references and body type annotations.
  If resolver value-signature metadata is incomplete and the collected
  signature/template is removed, body type-reference validation now skips that
  declaration instead of falling back to stale AST generic parameters.
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
  when resolver-restored parameter names differ from stale AST names, and it
  ignores stale same-name matches from different parameter positions.
- Resolver-backed behavior method collection now rebuilds behavior parameters
  from resolver-owned parameter names and types, so stale AST-only missing or
  extra parameters cannot distort impl conformance checks.
- Resolver-backed behavior method collection now also walks resolver-owned
  behavior method metadata in resolver order, so stale AST-only missing behavior
  methods cannot drop required methods from impl conformance checks.
- If resolver behavior-method metadata is incomplete and behavior collection is
  removed, default-body type-reference validation now skips that behavior
  instead of falling back to stale AST generic parameters.
- Typechecker resolver validation now derives behavior method display
  signatures and typed method metadata from one shared expectation pass, so the
  two resolver handoff checks cannot drift while scanning the same behavior
  method list.
- Typechecker resolver validation now carries value parameter count, names,
  display types, and typed parameter metadata through one shared parameter
  expectation and validation path, keeping those resolver value-signature
  handoff checks aligned.
- Typechecker resolver validation now carries value display-return and
  typed-return metadata through one shared return expectation and validation
  path, keeping those resolver value-signature handoff checks aligned.
- Typechecker resolver validation now carries value visibility and signature
  metadata through one expected value-symbol object, aligning value-symbol
  handoff checks with the type-like symbol expectation shape.
- Typechecker resolver validation now carries behavior visibility,
  type-parameter metadata, and method metadata through one expected
  behavior-symbol object, aligning behavior-symbol handoff checks with the
  value-symbol expectation shape.
- Typechecker resolver validation now names behavior method validation after
  the full display-signature and typed-metadata check it performs.
- Typechecker resolver validation now carries struct and enum visibility,
  type-parameter metadata, and kind-specific metadata through expected
  type-symbol objects, keeping type declaration handoff checks aligned.
- Typechecker resolver validation now carries enum variant owner, visibility,
  and payload metadata through one expected variant-symbol object, keeping
  variant handoff checks aligned.
- Typechecker resolver validation now carries behavior parent, impl, and
  required association list expectations as paired name/ref objects instead of
  parallel loose slices.
- Typechecker resolver validation now carries individual behavior association
  expectations as paired display-name and typed-ref objects instead of parallel
  loose values.
- Typechecker resolver validation now carries import source and visibility
  expectations through one expected import-symbol object.
- Typechecker resolver validation now carries module name, source, and
  visibility expectations through one expected module-symbol object.
- Typechecker resolver validation now carries local scope, mutability, source,
  and visibility expectations through one expected local-symbol object.
- Typechecker resolver validation now shares absent value-signature metadata
  checks for non-value resolver symbols while preserving per-kind diagnostics.
- Typechecker resolver validation now shares the remaining absent metadata
  emission path for module, import, and local resolver symbols while keeping
  per-kind diagnostic code tables local to each validator.
- Typechecker resolver validation now reuses the same absent metadata
  emission path across value, type-like, struct/enum, variant, and behavior
  resolver-symbol validators while preserving each validator's diagnostic
  codes.
- Typechecker resolver validation now shares absent source-metadata
  diagnostics across value, type-like, and variant resolver-symbol validators
  while preserving per-kind diagnostic codes.
- Typechecker resolver validation now centralizes resolver symbol presence
  diagnostic codes for missing declaration symbols, missing local symbols,
  extra declaration symbols, and extra local symbols.
- Typechecker resolver validation now centralizes source-mismatch diagnostic
  code bundles for module, stripped import, import, and local resolver symbols.
- Typechecker resolver validation now adapts type-parameter validation bundles
  into shared count diagnostics through the validation helper instead of
  constructing count diagnostics inline.
- Typechecker resolver expected value parameter construction now builds
  parameter names, display types, and typed AST types through one expected
  parameter constructor.
- Typechecker resolver expected return metadata construction now derives the
  default void return, display return, and typed AST return through one
  expected return constructor.
- Typechecker resolver expected type-parameter construction now pairs generic
  bound display metadata and typed bound-ref metadata through one expected
  type-parameter constructor.
- Typechecker resolver expected struct-field construction now pairs field
  display metadata and typed field metadata through one expected-field
  constructor.
- Typechecker resolver expected enum-variant payload construction now pairs
  optional payload display metadata and typed payload metadata through one
  expected-payload constructor.
- Typechecker resolver expected behavior-method construction now pairs display
  method signatures and typed method metadata through one expected-method
  constructor.
- Typechecker resolver expected value-signature construction now gathers
  parameter, return, and type-parameter expectations through one
  expected-signature constructor.
- Typechecker resolver expected value-symbol construction now pairs value
  signature expectations with visibility through one expected-symbol
  constructor.
- Typechecker resolver expected type-like symbol construction now pairs generic
  type-parameter expectations with optional visibility through one
  expected-type-like constructor.
- Typechecker resolver behavior-ref validation now separates role labels from
  per-check diagnostic code mappings, avoiding duplicated label bundles across
  contains and full-list checks.
- Typechecker resolver behavior-ref actual metadata selection now uses one
  role selector for parent, impl, and required refs instead of separate
  constructors for each association role.
- Typechecker resolver behavior-ref validation now asks the selected actual
  metadata to perform contains and full-list matching, keeping name/ref match
  semantics local to the resolver-owned metadata selection.
- Typechecker resolver behavior-ref owner restoration now splits exact
  behavior-key owner selection from the unique fallback owner path used when
  repairing stale AST association targets.
- Typechecker resolver expected behavior association construction now builds
  display names and typed refs through one expected-edge constructor.
- Typechecker resolver validation now stores expected behavior display
  signatures and typed method metadata as paired per-method expectations before
  deriving the resolver comparison lists.
- Typechecker resolver validation now stores expected behavior method
  expectations directly on behavior symbols instead of wrapping the per-method
  list before deriving resolver comparison lists.
- Typechecker resolver validation now stores expected struct field display and
  typed metadata as paired per-field expectations before deriving resolver
  comparison lists.
- Typechecker resolver validation now derives expected struct field counts from
  the per-field expectation list instead of storing a separate count.
- Typechecker resolver validation now stores expected struct field
  expectations directly on struct symbols instead of wrapping the per-field
  list before deriving resolver comparison lists.
- Typechecker resolver validation now stores expected enum variant payload
  display and typed metadata as one paired payload-type expectation.
- Typechecker resolver validation now derives expected enum variant payload
  counts from the paired payload-type expectation instead of storing a separate
  count.
- Typechecker resolver validation now names expected enum variant payload
  metadata after the paired payload-type expectation used by count, display,
  and typed checks.
- Typechecker resolver validation now stores expected type-parameter display
  bounds and typed bound refs as paired per-bound expectations before deriving
  resolver comparison lists.
- Typechecker resolver validation now stores expected type-parameter names and
  optional paired bounds as per-parameter expectations before deriving resolver
  comparison lists.
- Typechecker resolver validation now derives expected type-parameter counts
  from the per-parameter expectation list instead of storing a separate count.
- Typechecker resolver validation now stores expected type-parameter
  expectations directly on value and type-like symbols instead of wrapping the
  per-parameter list before deriving resolver comparison lists.
- Typechecker resolver validation now stores expected value parameter names,
  display types, and typed metadata as paired per-parameter expectations before
  deriving resolver comparison lists.
- Typechecker resolver validation now derives expected value parameter counts
  from the per-parameter expectation list instead of storing a separate count.
- Typechecker resolver validation now stores expected value parameter
  expectations directly on value signatures instead of wrapping the
  per-parameter list before deriving resolver comparison lists.
- Typechecker resolver validation now stores expected value display-return and
  typed-return metadata as one paired return expectation before deriving
  resolver comparison values.
- Typechecker resolver validation now constructs expected value typed-return
  metadata and its display name in the same return expectation helper.
- Typechecker resolver validation now stores expected value return metadata
  directly on value signatures instead of wrapping the paired display and typed
  return expectation.
- Typechecker resolver validation now derives and checks type-parameter
  counts, names, display bounds, and typed bound refs through shared
  expectation and validation paths for value and type-like symbols, reducing
  duplicate resolver metadata handoff construction.
- Typechecker resolver validation now derives struct field count, display
  metadata, and typed field metadata from one shared field expectation and
  validation path, keeping those resolver field handoff checks aligned.
- Typechecker resolver validation now derives enum variant payload count,
  display type, and typed payload metadata from one shared payload expectation
  and validation path, keeping those resolver variant handoff checks aligned.
- Resolver-backed struct and enum collection now also uses typed resolver
  generic bound refs, so generic type templates no longer retain stale AST-only
  behavior bounds after resolver validation.
- Struct field default expressions now participate in generic type-reference
  validation. Non-generic struct defaults are checked against their declared
  field type, struct literals inject omitted defaulted fields with concrete
  generic substitutions when needed, and resolver-backed validation skips those
  defaults when incomplete resolver field metadata has removed the collected
  struct instead of falling back to stale AST generic parameters.
- Resolver-backed struct and enum collection now shares resolver-restored
  behavior association ref handoff setup, keeping impl and required association
  metadata collection aligned before kind-specific field or variant metadata is
  rebuilt.
- Resolver-backed declaration collection now centralizes its temporary
  resolver-backed state toggling, so collection, impl/default restoration, and
  semantic validation use one scoped state helper instead of repeated manual
  flag flips.
- Resolver-backed behavior impl collection now shares the restored impl-block
  traversal used by impl method signature refresh and omitted default-method
  seeding, keeping resolver target restoration in one path.
- Resolver-backed type behavior-impl refresh now uses a shared restored
  struct/enum declaration traversal instead of open-coding the final
  type-name restoration pass.
- Resolver-backed behavior declaration collection now owns behavior name
  rekeying before restoring resolver-owned method and parent metadata.
- Resolver-backed value signature restoration now uses the same constructor
  helper pattern as resolver-restored struct, enum, and behavior metadata.
- Resolver-backed callable signature restoration now shares the function vs
  method key classifier between value metadata and generic template refresh.
- Resolver-backed method key restoration now also reuses that callable key
  classifier when matching resolver value symbols by declaration span.
- Method-key receiver parsing is now shared between resolver-backed method
  target restoration and generic method monomorphization inference.
- Resolver definition-span symbol lookup is now shared between callable
  signature restoration and impl target-name restoration.
- Resolver count validation now shares one diagnostic helper across value
  parameters, type parameters, struct fields, and enum variant payloads.
- Resolver metadata display fallbacks now share helpers for optional string and
  typed AST metadata diagnostics.
- Resolver optional AST type display now shares one helper for both `unknown`
  metadata and `none` payload diagnostics.
- Resolver string-list display now shares one helper across type-parameter,
  value-parameter, parameter-type, and variant-name diagnostics.
- Resolver comma-joined string rendering is now shared by resolver metadata
  lists and behavior-ref name diagnostics.
- Resolver named-list rendering is now shared by typed and display struct field
  metadata diagnostics.
- Resolver mapped-list rendering is now shared by AST type, type-parameter
  bound, behavior method, and behavior-ref metadata diagnostics.
- Resolver non-empty joined-list rendering is now shared by behavior-ref name
  and typed behavior-ref metadata diagnostics.
- Resolver behavior-ref pop and peek selection now share one helper across
  impl and required-association restoration paths.
- Resolver symbol metadata lookup is now shared by struct, enum, behavior, and
  behavior-ref restoration paths.
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
  Malformed nested generic type annotations inside explicit call type
  arguments stop before dependent call-signature checks.
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
  generic-receiver method, and UFC-style function call paths, and bound
  failures skip dependent specialization-body diagnostics.
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
- Resolver-backed behavior association validation now skips AST-only parent,
  impl, and required refs when resolver association metadata is missing, and
  clears stale impl associations before resolver-owned refresh.
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
- Resolver validation for `.extends(...)` parent type arguments now scopes the
  child behavior's generic parameters, so generic behavior inheritance such as
  `Pretty<T>.extends(Serializable<T>)` resolves before typechecker handoff.
  A paired resolver negative test rejects parent type arguments outside that
  child behavior parameter scope.
- Resolver/typechecker handoff also has `check_program_with_symbols` coverage
  for `.extends(...)` parent type arguments that reference the child behavior's
  generic parameters.
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
- Imported public generic top-level methods whose source module imports a
  generic enum dependency also compile and emit concrete call/definition pairs,
  covered by
  `tests/zen/multi_file_type_method_return_enum_dependency/main.zen`.
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
  Source-module imported generic enum dependencies for public generic
  `Type.impl` methods are covered by
  `tests/zen/multi_file_type_impl_return_enum_dependency/main.zen`.
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
- The CLI `check`, `emit`, `build`, and direct file paths now reject
  `build.zen` explicitly until deterministic build graph support exists, with
  integration coverage for the gated Phase 4 entrypoints.
- Resolver behavior-association validation now shares the parent/impl/requires
  name/ref diagnostic plumbing while keeping each path's explicit labels and
  error codes.
- Resolver behavior-association collection now shares resolver behavior-ref
  symbol lookup across parent, impl, and requires handoff paths.
- Resolver behavior-association validation now builds expected display names
  and typed refs in the same AST pass for impl/requires and parent edges.
- Resolver behavior-association expectation storage now uses a shared edge
  container for impl, requires, and parent validation inputs.
- Resolver behavior-association expectation storage now stores display names
  and typed refs as paired edges before deriving resolver comparison lists.
- Resolver behavior-association list validation now receives paired edge slices
  directly instead of wrapping the expected edge list.
- Resolver behavior-association expectation storage now names its paired
  storage and lookup after behavior edges rather than raw typed refs.
- Resolver behavior-association edge expectations now name their paired
  display string and typed metadata directly.
- Resolver behavior-association paired edge expectations now use edge naming
  consistently at the item and collection levels.
- Resolver behavior-association edge construction now uses the same edge
  terminology as the paired expectation type.
- Resolver value-signature expectation building now derives parameter names,
  display types, and typed metadata in one parameter pass.
- Resolver value-return expectation construction now uses metadata terminology
  consistently with the paired display and typed return expectation.
- Resolver value-parameter expectation construction now uses metadata
  terminology consistently with the paired display and typed parameter
  expectations.
- Resolver type-parameter expectation construction now uses metadata
  terminology consistently with paired bound display and typed bound-ref
  expectations.
- Resolver struct-field expectation construction now uses metadata terminology
  consistently with paired display and typed field expectations.
- Resolver enum variant payload expectation construction now uses metadata
  terminology consistently with paired display and typed payload expectations.
- Resolver behavior-method expectation construction now uses metadata
  terminology consistently with paired display-signature and typed method
  expectations.
- Resolver enum variant-name expectation construction now uses metadata
  terminology consistently with enum symbol variant-name metadata.
- Resolver type-like expectation construction now uses metadata terminology
  consistently with shared type-parameter and visibility metadata.
- Resolver value-signature expectation construction now uses metadata
  terminology consistently with parameter, return, and type-parameter metadata.
- Resolver-backed value declaration collection no longer seeds AST-only
  function or method signatures before resolver metadata restoration, while
  preserving generic template bodies for later metadata replacement.
- Resolver-backed top-level function collection now restores function names
  from resolver value symbols by declaration span, so stale AST-only function
  names cannot drop restored signatures or generic templates during setup.
- Resolver-backed struct, enum, and behavior collection now restores
  type-like names from resolver symbols by declaration span, so stale AST-only
  declaration names cannot drop restored fields, variants, behavior methods,
  or association metadata during setup.
- Resolver-backed impl-block collection no longer seeds AST-only method
  signatures before resolver metadata restoration, while preserving generic
  impl method template bodies for later metadata replacement.
- Resolver-backed `Type.impl` method collection now restores method keys by
  declaration span even when the AST-only impl target type name is stale.
- Resolver-backed method collection and generic `Type.impl` method templates
  now have coverage for restoring method keys by declaration span when both
  AST-only target type names and method names are stale.
- Resolver-backed behavior impl method collection and conformance now restore
  the impl target type name from resolver method symbols by declaration span,
  so stale AST-only impl target names cannot produce false undefined-type
  diagnostics.
- Resolver-backed behavior default synthesis also restores omitted-method impl
  targets from unique resolver behavior impl association refs when no explicit
  impl method span exists.
- Resolver-backed behavior default synthesis also restores omitted-method impl
  targets when AST-only behavior names are stale, using unique resolver impl
  association owners before resolver behavior ref restoration.
- Resolver-backed `.requires` validation now restores stale AST-only target
  type names from unique resolver required-behavior association refs.
- Resolver-backed `.requires` validation also restores stale target type names
  when AST-only behavior names or type arguments are stale, using unique
  resolver required association owners before declaration-order behavior ref
  restoration.
- Resolver-backed struct collection no longer seeds AST-only field metadata
  before resolver metadata restoration.
- Resolver-backed enum collection no longer seeds AST-only variant metadata
  before resolver metadata restoration.
- Resolver-backed behavior collection no longer keeps AST-only behavior method
  metadata when resolver method metadata is missing.
- Resolver-backed behavior collection no longer falls back to AST-only generic
  bounds when resolver bound metadata is missing.
- Resolver-backed value, struct, and enum collection now defaults missing
  resolver generic-bound metadata directly instead of carrying dead AST-bound
  fallback state.
- Resolver-backed value signature restoration now clears pre-seeded function,
  method, and generic template entries when resolver value metadata is missing
  or incomplete, so behavior impl collection and monomorphization cannot retain
  AST-only signatures.
- Behavior impl method collection now has coverage for clearing restored method
  keys after stale AST target/name repair when resolver value-signature metadata
  is incomplete.
- Resolver-backed `.requires` validation now restores stale AST target names
  from unique missing required-ref metadata before skipping incomplete resolver
  handoff, avoiding false diagnostics from stale AST-only required refs.
- Resolver-backed `.implements` validation and omitted-default synthesis now
  restore stale AST target names from unique missing impl-ref metadata before
  skipping incomplete resolver handoff, avoiding stale AST-only impl refs and
  default methods.
- Resolver-backed `.implements` and `.requires` target restoration now share
  the unique behavior-ref owner selection helper, reducing duplicate association
  handoff logic while preserving ambiguity checks.
- Resolver-backed type association collection now shares the same
  behavior-ref handoff helper for `.implements` and `.requires`, reducing
  duplicate resolver metadata setup while preserving incomplete-metadata
  tracking.
- Resolver behavior-method expectation building now reuses the shared
  value-signature metadata path for parameter names, display types, typed
  metadata, and returns.
- The docs truth gate now locks the quiet draft-PR CI trigger shape: no
  `pull_request.synchronize`, manual dispatch retained, and fmt/clippy/test
  jobs guarded by the draft-PR condition.
- Resolver variant payload expectations now pass the paired typed/display
  payload metadata directly to validation instead of wrapping it in a redundant
  intermediate object.
- Resolver field, enum variant-name, and behavior-method validators now borrow
  expectation slices instead of taking ownership of rebuilt expectation vectors.
- Resolver absent-metadata validation now shares a list-level helper for
  module, import, local, and variant symbol metadata entries.
- The same absent-metadata helper now covers the remaining type-like, kind,
  behavior, and value declaration validation paths.
- Resolver-backed declaration collection now restores type-parameter bounds
  through one shared resolver metadata helper for values, structs, enums, and
  behaviors.
- Resolver-backed declaration collection now restores type-parameter names
  through the same helper pattern across value, struct, enum, and behavior
  collection.
- AST declaration collection now uses the same type-parameter-name helper for
  behavior, struct, enum, function, method, and impl-method metadata.
- Generic template collection now uses one helper for local and imported
  function, method, and impl-method templates.
- AST callable metadata collection now uses one `FuncInfo` helper for local,
  impl, imported, and dependency function/method signatures.
- AST type metadata collection now uses shared struct and enum helpers across
  local declaration, module-graph import, and source-dependency seeding paths.
- AST behavior metadata collection now uses a shared helper across local
  declaration and module-graph import seeding paths.
- Behavior default method signature seeding now shares one helper across local
  and imported behavior implementation paths.
- Imported generic method template dependency attachment now uses the canonical
  source-module dependency bundle directly instead of a second wrapper type.
- Generic function templates now own source-module dependency attachment, so
  imported generic function and method templates share the same dependency path.
- Generic template dependency save/restore state now uses named dependency
  fields instead of a positional tuple across monomorphization.
- Generic template dependency save/restore now uses shared map helpers across
  structs, enums, functions, generic functions, methods, and generic methods.
- Generic function and method specialization now share missing type-argument
  inference diagnostics while preserving function/method wording.
- Generic function and method specialization now share the template-body
  save/check/restore path, with methods supplying only their receiver self type.
- Resolver-backed callable signature restoration now shares stale-entry cleanup
  and generic-template rekey helpers across function and method paths.
- Resolver-backed callable signature insertion now shares function-vs-method
  routing for restored value metadata.
- Resolver-backed generic template restoration now shares function-vs-method
  routing for restored generic value metadata.
- Resolver-backed struct and enum metadata restoration now use shared
  constructors for resolver type parameters, bounds, fields, and variants.
- Resolver-backed behavior metadata restoration now uses the same constructor
  pattern for resolver type parameters, bounds, and restored method signatures.
- Behavior implementation ref insertion now shares one helper across
  resolver-restored local impls and imported impl seeding.
- Resolver-backed behavior impl and requires target restoration now share one
  owner-selection helper for exact refs, unique refs, and missing-ref fallback.
- Behavior impl and requires validation now share resolver-ref override
  selection for restored behavior names and type arguments.
- Resolver behavior parent, impl, and requires validation now share metadata
  source selection for names and typed refs.
- Resolver type-parameter validation now shares one expected metadata bundle
  for counts, names, display bounds, and typed bound refs.
- Resolver type-parameter validation now carries name, display-bound, and
  typed-bound-ref message formatting through the validation bundle.
- Resolver count validation now carries count diagnostic message formatting
  through the shared count validation bundle.
- Resolver value-parameter validation now shares one expected metadata bundle
  for counts, names, display types, and typed AST types.
- Resolver value-parameter validation now carries name, display-type, and
  typed-type message formatting through a validation bundle.
- Resolver value return-type validation now carries display and typed return
  message formatting through a validation bundle.
- Resolver value return-type validation now owns its resolver diagnostic code
  mapping instead of constructing those codes at the call site.
- Resolver behavior-association list validation now shares one expected
  metadata bundle for display names and typed refs.
- Resolver behavior-association validation now uses one role/check mapping for
  parent, impl, and requires diagnostic metadata.
- Resolver behavior-association validation now uses the same role mapping for
  parent, impl, and requires resolver metadata selection.
- Resolver behavior-association contains/list validation now shares role-aware
  wrapper helpers for parent, impl, and requires diagnostics.
- Resolver behavior-association diagnostics now carry contains/list name/ref
  message formatting through the behavior-ref validation bundle.
- Resolver struct-field validation now shares one expected metadata bundle
  for counts, display fields, and typed AST fields.
- Resolver struct-field validation now carries display-field and typed-field
  message formatting through a validation bundle.
- Resolver struct-field validation now owns its resolver diagnostic code
  mapping instead of constructing those codes at the call site.
- Resolver behavior-method validation now shares one expected metadata bundle
  for display signatures and typed method metadata.
- Resolver behavior-method validation now carries display-method and
  typed-method message formatting through a validation bundle.
- Resolver behavior-method validation now owns its resolver diagnostic code
  mapping instead of constructing those codes at the call site.
- Resolver variant-payload validation now shares one expected metadata bundle
  for counts, display payload types, and typed AST payloads.
- Resolver variant-payload validation now carries display-payload and
  typed-payload message formatting through a validation bundle.
- Resolver variant-payload validation now owns its resolver diagnostic code
  mapping instead of constructing those codes at the call site.
- Resolver variant owner-name validation now carries its diagnostic code and
  message formatting through a validation bundle.
- Resolver variant owner-name validation now owns its resolver diagnostic code
  mapping instead of constructing that code at the call site.
- Resolver variant-name validation now carries its diagnostic code and message
  formatting through a validation bundle.
- Resolver variant-name validation now owns its resolver diagnostic code
  mapping instead of constructing that code at the call site.
- Resolver visibility validation now shares one diagnostic helper across
  module, import, local, type-like, variant, and value symbols.
- Resolver visibility validation now carries its diagnostic code and display
  formatting through a validation bundle.
- Resolver visibility validation now also owns its full diagnostic message
  formatting, matching the source/count validation helper shape.
- Resolver module visibility validation now owns its resolver diagnostic code
  mapping instead of constructing that code at the call site.
- Resolver import visibility validation now owns its resolver diagnostic code
  mapping instead of constructing that code at its call sites.
- Resolver type-like visibility validation now owns its resolver diagnostic code
  mapping instead of constructing that code at the call site.
- Resolver variant visibility validation now owns its resolver diagnostic code
  mapping instead of constructing that code at the call site.
- Resolver value visibility validation now owns its resolver diagnostic code
  mapping instead of constructing that code at the call site.
- Resolver local visibility validation now owns its resolver diagnostic code
  mapping instead of constructing that code at the call site.
- Resolver source validation now shares one diagnostic helper across module,
  import, and local symbols.
- Resolver source validation now carries source diagnostic message formatting
  through the shared source validation bundle.
- Resolver type-like source absence validation now owns its resolver diagnostic
  code mapping instead of constructing that code at the call site.
- Resolver variant source absence validation now owns its resolver diagnostic
  code mapping instead of constructing that code at the call site.
- Resolver value source absence validation now owns its resolver diagnostic
  code mapping instead of constructing that code at the call site.
- Resolver local mutability validation now shares the same diagnostic helper
  shape used by the resolver metadata validation paths.
- Resolver module mutability absence validation now owns its resolver diagnostic
  code mapping instead of constructing that code at the call site.
- Resolver import mutability absence validation now owns its resolver diagnostic
  code mapping instead of constructing that code at the call site.
- Resolver type-like mutability absence validation now owns its resolver
  diagnostic code mapping instead of constructing that code at the call site.
- Resolver variant mutability absence validation now owns its resolver
  diagnostic code mapping instead of constructing that code at the call site.
- Resolver value mutability absence validation now owns its resolver diagnostic
  code mapping instead of constructing that code at the call site.
- Resolver local mutability validation now carries its diagnostic code and
  display formatting through a validation bundle.
- Resolver local mutability validation now owns its resolver diagnostic code
  mapping instead of constructing that code at the call site.
- Resolver local mutability validation now also owns its full diagnostic
  message formatting, matching the source/count/visibility helper shape.
- Resolver extra-symbol validation now shares one diagnostic helper across
  declaration/import/module and local symbol checks.
- Resolver missing-symbol validation now shares one diagnostic helper across
  declaration/import/module/type/behavior/variant/value and local checks.
- Resolver extra- and missing-symbol validation now push diagnostics through
  one shared presence helper.
- Resolver extra- and missing-symbol validation now share one presence
  validation bundle for diagnostic codes and message formatting.
- Resolver absent-source metadata validation now reuses the shared resolver
  source diagnostic helper.
- Resolver absent-source metadata validation now lets the validation bundle
  build its source diagnostic configuration.
- Resolver stripped-import validation now reuses the shared resolver
  visibility and source diagnostic helpers.
- Resolver absent value-signature metadata validation now reuses the shared
  absent-metadata entry helper.
- Resolver absent-metadata entry validation now carries diagnostic message
  formatting through an explicit absent metadata entry object.
- Resolver absent-metadata validation bundles now build typed absent metadata
  entries directly, so the validation path no longer rewraps raw
  present/code/label tuples before emitting diagnostics.
- Resolver absent value-signature detail validation now shares the same helper
  across module, import, local, type-like, and variant symbols.
- Resolver absent value-signature metadata validation now lets the validation
  bundle build its parameter and return metadata entries.
- Resolver module value-signature absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver import value-signature absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver local value-signature absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver type-like value-signature absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver variant value-signature absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver absent type-parameter metadata validation now shares one helper
  across module, import, local, and variant symbols.
- Resolver absent type-parameter metadata validation now lets the validation
  bundle build its count/name/bounds metadata entries.
- Resolver absent field metadata validation now shares one helper across
  module, import, local, enum, variant, behavior, and value symbols.
- Resolver absent field metadata validation now lets the validation bundle
  build its count/display/typed field metadata entries.
- Resolver absent variant metadata validation now shares one helper across
  module, import, local, struct/type, behavior, and value symbols.
- Resolver absent variant metadata validation now lets the validation bundle
  build its names/owner/payload metadata entries.
- Resolver absent behavior-association metadata validation now shares one
  helper across module, import, local, variant, behavior, and value symbols.
- Resolver absent behavior-association metadata validation now lets the
  validation bundle build its impl/requires metadata entries.
- Resolver module behavior-association absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver import behavior-association absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver local behavior-association absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver variant behavior-association absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver behavior-symbol behavior-association absence validation now owns its
  resolver diagnostic code mapping instead of constructing those codes at the
  call site.
- Resolver value behavior-association absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver absent behavior-declaration metadata validation now shares one
  helper across module, import, local, variant, and value symbols.
- Resolver absent behavior-declaration metadata validation now lets the
  validation bundle build its method/parent metadata entries.
- Resolver module behavior-declaration absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver import behavior-declaration absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver local behavior-declaration absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver variant behavior-declaration absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver value behavior-declaration absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver absent mutability metadata validation now shares one helper across
  module, import, type-like, variant, and value symbols.
- Resolver absent mutability metadata validation now lets the validation bundle
  build its mutability metadata entry.
- Generic type substitution now covers raw pointers, arrays, and function
  signatures so Phase 5 specializations do not leave nested type parameters
  inside composite type shapes.
- Generic function-type substitutions now round-trip through nested generic
  type arguments instead of degrading to `void`.
- Generic method call arity diagnostics now preserve method wording through
  the shared call-signature checker.
- Explicit generic function and method type-argument arity failures now stop
  before specialization emits misleading follow-up inference diagnostics.
- Invalid explicit generic function and method type-argument arity now also
  skips dependent signature checks so bare omitted type parameters do not
  cascade into argument or return mismatches.
- Malformed nested generic type annotations inside explicit function and
  method call type arguments now also skip dependent signature checks.
- Generic behavior bound failures now skip dependent function and method body
  specialization diagnostics.
- Resolver value-parameter validation now owns its resolver diagnostic code
  mapping instead of constructing those codes at the call site.
- Resolver module type-parameter absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver import type-parameter absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver local type-parameter absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver variant type-parameter absence validation now owns its resolver
  diagnostic code mapping instead of constructing those codes at the call site.
- Resolver module field absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver import field absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver local field absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver type-like field absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver variant field absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver behavior field absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver value field absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver module variant absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver import variant absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver local variant absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver type-like variant absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver behavior variant absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver value variant absence validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver type-like type-parameter validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver value type-parameter validation now owns its resolver diagnostic
  code mapping instead of constructing those codes at the call site.
- Resolver value parameter-count validation now owns its resolver diagnostic
  code mapping instead of constructing that code at the call site.
- Resolver field-count validation now owns its resolver diagnostic code mapping
  instead of constructing that code at the call site.
- Resolver variant payload-count validation now owns its resolver diagnostic
  code mapping instead of constructing that code at the call site.

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
