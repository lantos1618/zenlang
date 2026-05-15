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
  declarations or definitions as calls. The generated-C specialization test also
  scans covered underscore-style generated calls and fails if any call lacks an
  emitted definition, and every executable integration fixture runs the same
  generated-call scan before compiling C. The scan itself is guarded by
  `integration::generated_c_call_definition_scan_reports_missing_generated_calls`.
  Worklist dedup checks count generated function definitions directly, guarded
  by `integration::generated_c_definition_count_ignores_prototypes`.
- Generic method specialization preserves concrete `Self` receiver context in
  call-site typing and specialized method bodies for generic struct and enum
  receivers, with executable and generated-C coverage in
  `tests/zen/generic_method_self.zen`, including receiver-based inference for
  `Self`-only generic method signatures and nested
  `Box<Option<i32>>` specialization dependency ordering.
- Generic method specializations that call generic functions have executable
  and generated-C coverage in `tests/zen/generic_method_worklist.zen`, including
  call-resolution assertions for the reached generic function dependency.
  Imported public generic methods can also specialize private source-module
  generic helper calls without exposing those helpers to entry modules, covered
  by `tests/zen/multi_file_type_method_worklist/main.zen` and
  `integration::imported_type_method_worklist_helpers_are_not_directly_visible`.
  The same scoped dependency mechanism covers private source-module generic
  method helpers through
  `tests/zen/multi_file_type_method_method_dependency/main.zen` and
  `integration::imported_type_method_dependencies_are_not_directly_visible`.
  Source-module imports used by public imported generic method templates are
  covered by `tests/zen/multi_file_type_method_imported_dependency/main.zen`
  and
  `integration::imported_type_method_imported_dependencies_are_not_directly_visible`.
  Imported public generic non-behavior `Type.impl` method templates that use
  source-module imported generic types and methods are covered by
  `tests/zen/multi_file_type_impl_imported_type_dependency/main.zen` and
  `integration::imported_type_impl_imported_type_dependencies_are_not_directly_visible`.
  Imported public generic function templates that use source-module imported
  generic types and methods are covered by
  `tests/zen/multi_file_generic_imported_type_dependency/main.zen` and
  `integration::imported_generic_function_imported_type_dependencies_are_not_directly_visible`.
  Transitive imported generic helper templates also carry their own private
  source-module dependencies during specialization, covered by
  `tests/zen/multi_file_generic_imported_transitive_dependency/main.zen` and
  `integration::imported_generic_function_transitive_dependencies_are_not_directly_visible`.
- Resolver method value symbols carry complete value-signature metadata, and
  typechecker setup rejects method signature drift before method body
  collection. Function-typed method parameters and returns are included in
  that handoff coverage.
- Resolver-backed generic type-reference validation reads collected
  resolver-restored function and method signatures, so stale AST-only parameter
  or return annotations cannot produce false unknown-type diagnostics.
- Resolver-backed top-level method collection also restores method names from
  resolver value symbols by declaration span when AST-only method names are
  stale, so collected `Type.method` signatures use resolver-owned names.
- Resolver-backed non-behavior `Type.impl` method collection restores method
  names from resolver value symbols by declaration span when AST-only impl
  method names are stale, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_method_name_metadata`.
  Generic `Type.impl` method templates also restore resolver-owned method names,
  parameters, and returns, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_name_metadata`.
- Module graph typechecking now seeds graph imports and then runs
  resolver-backed declaration collection for the current module, so multi-file
  graph modules use the same resolver-owned metadata handoff as
  `check_program_with_symbols` instead of returning to plain AST declaration
  collection after resolver validation.
- Resolver-backed struct and enum collection shares the resolver-restored
  behavior association ref setup before rebuilding kind-specific metadata,
  keeping impl and required association handoffs on the same collection path.
- Resolver-backed declaration collection uses one scoped state helper for
  collection, impl/default restoration, and semantic validation instead of
  repeated manual resolver-backed flag toggles.
- Resolver-backed behavior impl method signature refresh and omitted
  default-method seeding now share the same restored impl-block traversal.
- Resolver-backed type behavior-impl refresh now shares a restored struct/enum
  declaration traversal for the final type-name restoration pass.
- Resolver-backed behavior declaration collection now centralizes behavior
  name rekeying before method and parent metadata restoration.
- Resolver-backed value signature restoration now constructs `FuncInfo`
  through a dedicated resolver metadata helper, aligning callable metadata with
  resolver-restored type and behavior constructors.
- Resolver-backed callable signature restoration now shares one function vs
  method key classifier for concrete value metadata and generic templates.
- Resolver-backed method key restoration now uses the same callable key
  classifier for resolver value-symbol declaration-span matching.
- Method-key receiver parsing is shared between resolver-backed method target
  restoration and generic method monomorphization inference, covered by
  `typechecker::tests::method_signature_key_helpers_share_receiver_parsing`.
- Resolver definition-span symbol lookup is shared between callable signature
  restoration and impl target-name restoration, covered by
  `typechecker::tests::resolver_symbol_lookup_helpers_share_definition_span_fallbacks`.
- Resolver count validation now shares one diagnostic helper across value
  parameters, type parameters, struct fields, and enum variant payloads, with
  display coverage in
  `typechecker::tests::resolver_count_display_formats_known_and_missing_counts`.
- Resolver metadata display fallbacks now share helpers for optional string and
  typed AST metadata diagnostics, covered by
  `typechecker::tests::resolver_metadata_display_formats_known_and_missing_values`.
- Resolver optional AST type display now shares one helper for both `unknown`
  metadata and `none` payload diagnostics, covered by the same display
  fallback test.
- Resolver string-list display now shares one helper across type-parameter,
  value-parameter, parameter-type, and variant-name diagnostics, covered by
  `typechecker::tests::resolver_string_list_display_formats_known_and_missing_lists`.
- Resolver comma-joined string rendering is now shared by resolver metadata
  lists and behavior-ref name diagnostics, covered by the same string-list
  display test.
- Resolver named-list rendering is now shared by typed and display struct field
  metadata diagnostics, covered by
  `typechecker::tests::resolver_named_list_display_formats_known_and_missing_items`.
- Resolver mapped-list rendering is now shared by AST type, type-parameter
  bound, behavior method, and behavior-ref metadata diagnostics, covered by
  `typechecker::tests::resolver_display_list_formats_mapped_known_and_missing_items`.
- Resolver non-empty joined-list rendering is now shared by behavior-ref name
  and typed behavior-ref metadata diagnostics, covered by
  `typechecker::tests::resolver_nonempty_joined_list_formats_present_empty_and_missing_items`.
- Resolver behavior-ref pop and peek selection now share one helper across
  impl and required-association restoration paths, covered by
  `typechecker::tests::resolver_behavior_ref_helpers_share_pop_and_peek_selection`.
- Resolver absent mutability metadata validation now uses a validation bundle
  to build the shared mutability metadata entry, covered by
  `typechecker::tests::mutability_absence_validation_builds_entry`.
- Resolver symbol metadata lookup is now shared by struct, enum, behavior, and
  behavior-ref restoration paths, covered by
  `typechecker::tests::resolver_symbol_metadata_helper_requires_symbol_and_selected_metadata`.
- Resolver-backed generic type-reference validation also derives scoped generic
  type parameters and struct, enum, behavior, and impl-method declaration type
  references from collected resolver-restored metadata, so stale AST-only
  generic parameter names cannot produce false unknown-type diagnostics.
- Resolver-backed generic bound validation defers AST-only type-parameter
  constraint checks until resolver metadata has been restored for functions,
  structs, enums, behaviors, and impl methods, so stale AST-only behavior
  constraints cannot produce false generic-bound diagnostics.
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
  direct diagnostics for the conflicting parameter and concrete types,
  including receiver-derived generic method type arguments that conflict with
  later call arguments.
  Generic inference also walks function, array, and raw-pointer parameter
  shapes, so nested type parameters inside compound arguments can produce
  direct conflict diagnostics.
  Resolver rejects duplicate generic type-parameter names across value, type,
  and behavior declarations, covered by
  `resolver_phase2::resolver_rejects_duplicate_type_parameter_names`.
  Generic behavior bound failures are covered across direct function, method,
  generic-receiver method, and UFC-style function call paths.
- Typechecker setup accepts resolver `SymbolTable` through
  `check_program_with_symbols`.
- Resolver behavior impl method symbols carry complete value-signature metadata,
  and typechecker setup rejects impl-method signature drift before behavior impl
  body collection. Function-typed impl-method parameters and returns are
  included in that handoff coverage.
- Behavior impl conformance reads the collected `Type.method` signature,
  including resolver-restored impl-method metadata, so stale AST-only method
  signatures cannot produce false impl diagnostics.
- Resolver-backed behavior impl conformance also restores impl method names
  from resolver-owned value symbols when AST-only impl method names are stale,
  without masking real extra impl methods that lack resolver-owned required
  method symbols.
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
- Omitted behavior default methods now refresh their method-table signatures
  from validated resolver behavior method metadata, so function-typed default
  methods do not retain stale AST-only signatures after behavior collection.
- Typechecker resolver validation now derives behavior method display
  signatures and typed method metadata from one shared expectation pass, keeping
  those behavior-method handoff checks aligned.
- Typechecker resolver validation now carries value parameter count, names,
  display types, and typed parameter metadata through one shared parameter
  expectation and validation path, keeping value-signature handoff checks
  aligned.
- Typechecker resolver validation now carries value display-return and
  typed-return metadata through one shared return expectation and validation
  path, keeping value-signature handoff checks aligned.
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
  expectation and validation paths for value and type-like symbols, keeping
  generic metadata handoff checks aligned.
- Typechecker resolver validation now derives struct field count, display
  metadata, and typed field metadata from one shared field expectation and
  validation path, keeping resolver field handoff checks aligned.
- Typechecker resolver validation now derives enum variant payload count,
  display type, and typed payload metadata from one shared payload expectation
  and validation path, keeping resolver variant handoff checks aligned.
- Resolver-backed behavior default synthesis now waits until resolver behavior
  and impl-method metadata has been restored, and it treats resolver-restored
  impl method names as explicit overrides. This prevents stale AST-only impl
  method names from causing default methods to overwrite explicit impl
  signatures, covered by
  `typechecker::tests::collect_declarations_with_symbols_skips_default_when_resolver_restores_impl_method_name`.
- Resolver-backed behavior default synthesis also uses the resolver-owned
  behavior impl ref when AST-only impl behavior names or type arguments are
  stale, so omitted defaults are synthesized from the validated behavior
  association, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_impl_behavior_for_defaults`.
- Resolver-backed declaration collection now defers impl/requires semantic
  checks until after resolver value and behavior metadata has been restored, so
  stale AST-only behavior signatures cannot produce false impl diagnostics.
- Typechecker setup rejects resolver local mutability mismatches before
  collecting typed bodies from the AST.
- Typechecker setup rejects resolver local visibility/source mismatches before
  collecting typed bodies from the AST.
- Typechecker setup rejects resolver local parameter-count and return-type
  metadata before collecting typed bodies from the AST.
- Typechecker setup rejects resolver local display and typed type, field,
  variant, and behavior metadata before collecting typed bodies from the AST.
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
- Typechecker setup rejects resolver import binding display and typed type,
  field, variant, and behavior metadata before seeding imported module-call
  bindings.
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
- Typechecker setup rejects resolver module display and typed type, field,
  variant, and behavior metadata before validating imported binding symbols.
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
- Resolver rejects duplicate parameter names in behavior method signatures
  before typechecker metadata collection, covered by
  `resolver_phase2::resolver_rejects_duplicate_signature_parameter_names`.
- Resolver type symbols carry behavior impl and `.requires` association
  metadata, and typechecker setup rejects missing or extra association metadata
  before collecting behavior impls/requires from the AST. Specialized behavior
  references such as `Json<str>` are included in this resolver handoff
  validation.
- Resolver-backed `.requires` conformance checks read validated resolver
  required-behavior refs, so stale AST-only required behavior type arguments
  cannot produce false missing-impl diagnostics.
- Resolver-backed `.implements` conformance checks read validated resolver
  behavior impl refs before method conformance, so stale AST-only impl behavior
  type arguments cannot produce false method signature diagnostics.
- Resolver-backed `.implements` and `.requires` conformance also falls back to
  declaration-order resolver refs when AST-only behavior names are stale, so
  validated resolver behavior associations cannot be shadowed by stale AST names
  during semantic checks.
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
- Generic behavior declarations enforce their own type-parameter bounds when
  concrete behavior type arguments are instantiated, covered by positive and
  negative typechecker tests for `Serializable<T: Json<T>>`.
- Generic behavior inheritance accepts parent type arguments that reference the
  child behavior's own type parameters, deferring those bound checks until a
  concrete behavior specialization is instantiated.
- UFCS dispatch through substituted generic behavior bounds is covered by
  `tests/zen/behavior_json_generic_bound_ufcs.zen` plus generated-C assertions
  that reject unresolved `T_encode` calls.
- Imported public types carry source-module behavior impl associations and impl
  methods into graph-owned generic behavior-bound dispatch, covered by
  `tests/zen/multi_file_imported_behavior_impl/main.zen`. Private
  source-module behavior impls are not exported as direct methods on imported
  public types.
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
- Imported public generic functions preserve behavior bounds whose behavior was
  imported by the source module, covered by
  `tests/zen/multi_file_imported_function_imported_behavior_bound/main.zen`.
- Imported public function parameter and return-type dependencies preserve
  behavior impl associations even when the entry module imports only the
  functions, covered by
  `tests/zen/multi_file_imported_function_param_type_dependency/main.zen`,
  `tests/zen/multi_file_imported_function_return_type_dependency/main.zen`, and
  `tests/zen/multi_file_imported_function_imported_return_type_behavior/main.zen`;
  `integration::imported_function_signature_type_dependencies_are_not_directly_visible`
  verifies the dependency type is not directly constructible without an
  entry-module import.
- Imported public generic functions preserve source-module imported generic
  enum return dependencies, covered by
  `tests/zen/multi_file_imported_generic_function_return_enum_dependency/main.zen`.
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
  Resolver rejects duplicate local `.implements` edges before recording
  duplicate metadata, covered by
  `resolver_phase2::resolver_rejects_duplicate_behavior_impl_edges`.
  Resolver rejects duplicate local `.requires` edges before recording duplicate
  metadata, covered by
  `resolver_phase2::resolver_rejects_duplicate_behavior_required_edges`.
- Behavior inheritance `.extends` is parsed, resolved against known behaviors,
  and typechecked so child behavior impls must satisfy inherited parent methods
  while duplicate edges, cyclic inheritance, and conflicting inherited method
  signatures are rejected.
  Resolver rejects duplicate local parent edges before recording duplicate
  metadata, covered by
  `resolver_phase2::resolver_rejects_duplicate_behavior_parent_edges`.
- Concrete generic behavior parent inheritance, such as
  `PrettyJson.extends(Json<str>)`, is parsed, recorded in resolver metadata,
  checked with substituted parent method signatures, and covered by
  `tests/zen/behavior_generic_parent_inheritance.zen`.
- Generic behavior inheritance parent arguments may reference the child
  behavior's own type parameters, such as `Pretty<T>.extends(Serializable<T>)`.
  This is covered at resolver level by
  `resolver_phase2::resolver_accepts_behavior_parent_type_args_from_child_type_params`,
  guarded negatively by
  `resolver_phase2::resolver_rejects_behavior_parent_type_args_outside_child_type_params`,
  and checked through resolver/typechecker handoff by
  `typechecker::tests::check_program_with_symbols_accepts_resolver_behavior_parent_child_type_param_refs`.
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
- Non-behavior `Type.impl = { ... }` blocks parse, resolve as `Type.method`
  value symbols, typecheck, and emit concrete method functions, including
  generic impl methods and graph-owned public imports, preventing silently
  ignored impl method bodies. Imported private impl methods remain inaccessible
  through graph-owned imports. Resolver rejects duplicate non-behavior impl
  method symbols, including collisions with top-level `Type.method`
  declarations.
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
  path, including end-to-end generated-C coverage for imported public generic
  top-level methods. Imported method templates carry source-module private
  generic function and method helper dependencies, including source-module
  imports, only during specialization. Imported public generic non-behavior
  `Type.impl` method templates also carry source-module imported generic type
  and method dependencies only during specialization, while private imported
  methods, helper functions, and helper types remain inaccessible from entry
  modules.
- The resolver-backed typechecker path collects function and method signatures
  from validated resolver value symbols, including typed function-signature
  metadata, reducing duplicate AST-only declaration collection for value
  signatures.
- Resolver-backed typechecker collection updates generic function and generic
  method templates with validated resolver type-parameter, bound-ref,
  parameter-type, and return metadata, so monomorphization templates no longer
  retain stale AST-only generic names, bounds, or function-type signatures after
  resolver validation.
- Resolver-backed value signature cleanup removes generic function and method
  templates when resolver value metadata is missing or incomplete, so incomplete
  handoff cannot leave stale monomorphization templates behind.
- Behavior impl method collection clears restored method keys after stale
  AST-only target/name repair when resolver value-signature metadata is
  incomplete, so partial handoff cannot retain stale impl method signatures.
- Resolver-backed `.requires` validation restores stale AST target names from a
  unique missing required-ref owner before skipping incomplete resolver handoff,
  so stale AST-only required refs cannot produce false undefined-type errors.
- Resolver-backed `.implements` validation and omitted-default synthesis restore
  stale AST target names from a unique missing impl-ref owner before skipping
  incomplete resolver handoff, so stale AST-only impl refs cannot synthesize
  default methods or produce false undefined-type errors.
- Resolver-backed `.implements` and `.requires` target restoration share a
  unique behavior-ref owner helper, reducing duplicate association handoff logic
  while preserving ambiguity checks.
- Resolver-backed type association collection shares the same behavior-ref
  handoff helper for `.implements` and `.requires`, reducing duplicate resolver
  metadata setup while preserving incomplete-metadata tracking.
- Resolver-backed generic template collection also derives return-type presence
  from validated resolver metadata, so stale AST-only missing return annotations
  cannot erase resolver-owned generic function or method returns before
  monomorphization.
- Resolver-backed generic template collection rebuilds template parameters from
  validated resolver parameter names and types, so stale AST-only parameter
  counts cannot leave monomorphization templates with missing or extra
  parameters.
  Parameter mutability is preserved by positional fallback when restored
  resolver parameter names differ from stale AST names, covered by
  `typechecker::tests::collect_declarations_with_symbols_preserves_generic_template_param_mutability_by_position`.
- Resolver-backed behavior method collection rebuilds behavior parameters from
  resolver-owned parameter names and types, so stale AST-only missing or extra
  parameters cannot distort impl conformance checks.
- Resolver-backed behavior method collection also walks resolver-owned behavior
  method metadata in resolver order, so stale AST-only missing behavior methods
  cannot drop required methods from impl conformance checks.
- Resolver-backed struct and enum collection uses typed resolver generic bound
  refs, so generic type templates no longer retain stale AST-only behavior
  bounds after resolver validation.
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
- Resolver value symbols carry display and typed-ref generic type-parameter
  bounds, and typechecker setup rejects mismatches before collecting function or
  method metadata.
- Typechecker setup rejects resolver value source, display and typed field,
  variant, behavior, and mutability metadata before collecting function or
  method metadata.
- Typechecker setup rejects resolver type and behavior source, display and
  typed value-signature metadata, and mutability metadata before collecting
  declaration metadata.
- Typechecker setup rejects resolver struct display and typed variant metadata
  and resolver enum display and typed field metadata before collecting
  declaration metadata.
- Typechecker setup rejects resolver behavior display and typed field, variant,
  impl, and required-behavior metadata before collecting behavior metadata.
- Typechecker setup rejects resolver variant import, display and typed value,
  generic, field, enum-type, behavior, and mutability metadata before
  collecting enum variant metadata.
- Resolver type and behavior symbols carry generic parameter-count metadata, and
  typechecker setup rejects mismatches before collecting declaration metadata.
- Resolver type and behavior symbols carry generic parameter-name metadata, and
  typechecker setup rejects mismatches before collecting declaration metadata.
- Resolver type and behavior symbols carry generic type-parameter bounds, and
  typechecker setup rejects mismatches before collecting declaration metadata.
- Typechecker setup rejects generic behavior type-parameter bound drift,
  including bounds with type arguments such as `T: Json<T>`.
- Resolver symbols carry typed behavior association metadata for `.extends`,
  `.implements`, and `.requires`, and resolver-backed typechecker collection
  uses that structured metadata for inherited parents and behavior impls,
  reducing duplicate AST-only behavior association reconstruction.
- Typechecker setup validates typed resolver behavior association refs for
  generic parents, impls, and required-behavior assertions, so matching
  display-name metadata cannot hide structured association drift.
- Resolver-backed behavior association collection skips AST-only parent, impl,
  and required refs when resolver association metadata is missing, so stale AST
  associations cannot survive incomplete resolver handoff.
- Resolver-backed behavior inheritance checks validate restored resolver parent
  refs before cycle and method-coherence checks, so stale AST-only parent names
  or type arguments cannot leak false extends diagnostics.
- Resolver type symbols carry visibility metadata for structs and enums, and
  typechecker setup rejects mismatches before collecting declaration metadata.
- Typechecker setup rejects resolver behavior symbol visibility mismatches
  before collecting behavior metadata.
- Resolver behavior symbols carry method signature metadata, and typechecker
  setup rejects mismatches before collecting behavior metadata.
- Resolver behavior method signature metadata preserves function-typed
  parameters and returns, and typechecker setup rejects function-type method
  signature drift before behavior metadata collection.
- Typechecker setup validates resolver typed behavior method metadata in
  addition to display signatures, so stale `behavior_method_types` cannot feed
  resolver-backed behavior collection behind matching signature strings.
- The resolver-backed typechecker path collects behavior method signatures from
  validated resolver behavior symbols, including typed function-method metadata,
  reducing duplicate AST-only declaration collection for behavior method
  signatures.
- Resolver-backed behavior method collection also restores method names from
  validated resolver metadata, so stale AST-only behavior method names cannot
  shadow resolver-owned signatures during impl conformance.
- Resolver-backed behavior method collection derives return-type presence from
  validated resolver metadata, so stale AST-only missing return annotations
  cannot erase resolver-owned behavior method returns.
- Resolver behavior method signature metadata preserves generic return types on
  generic behaviors, and typechecker setup rejects generic method-signature
  handoff drift before behavior metadata collection.
- Resolver behavior method signature metadata preserves function-typed
  parameters and returns over generic type parameters, and typechecker setup
  rejects generic function-type method drift before behavior metadata
  collection.
- Resolver top-level method and behavior impl method value symbols preserve
  function-typed parameters and returns, and typechecker setup rejects
  function-type method handoff drift before method body collection.
- Typechecker setup validates typed resolver value-signature metadata in
  addition to display signatures, so stale `parameter_types` or `return_type`
  cannot feed resolver-backed value collection behind matching signature
  strings.
- Resolver struct symbols carry field-count metadata, and typechecker setup
  rejects mismatches before collecting struct field metadata.
- Resolver struct symbols carry field-name/type metadata, and typechecker setup
  rejects mismatches before collecting struct field metadata.
- Resolver struct field metadata preserves function-typed fields, and
  typechecker setup rejects function-type field drift before struct metadata
  collection.
- Typechecker setup validates typed resolver struct field metadata in addition
  to display strings, so stale `field_types` cannot feed resolver-backed
  struct collection behind matching field names.
- The resolver-backed typechecker path collects struct field metadata from
  validated resolver type symbols, including typed function-field metadata,
  reducing duplicate AST-only declaration collection for struct fields.
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
- Typechecker setup validates typed resolver enum payload metadata in addition
  to display strings, so stale `variant_payload_type` cannot feed
  resolver-backed enum collection behind matching payload names.
- The resolver-backed typechecker path collects enum variant payload metadata
  from validated resolver variant symbols, including typed function-payload
  metadata, reducing duplicate AST-only declaration collection for enum
  variants.
- Resolver enum variant payload metadata preserves function-typed payloads over
  generic type parameters, and typechecker setup rejects generic function-type
  payload drift before enum variant metadata collection.
- CLI `check`, `emit`, `build`, and direct file invocation reject `build.zen`
  explicitly until deterministic build graph support exists, with integration
  coverage for the gated Phase 4 entrypoints.
- Multi-file generic import fixtures cover imported generic enum/function
  specialization through C generation and runtime execution, including generated
  C assertions that imported mangled calls have matching concrete definitions.
  Imported generic function templates carry source-module imported generic type
  and method dependencies only during specialization, while helper types remain
  inaccessible from entry modules. Transitive imported generic helper
  templates carry their own private generic dependencies during specialization
  without exposing those helpers to entry modules.
- Multi-file generic behavior-bound fixtures cover imported public behavior
  declarations through module-graph resolver validation, typechecking, C
  generation, and runtime execution.
- Multi-file behavior inheritance fixtures cover imported behavior parent edges
  through module-graph typechecking, including direct and transitive negative
  missing-method diagnostics and generated-C assertions for a positive
  inherited-bound specialization.
- Resolver-backed behavior association validation rejects extra typed behavior
  parent, impl, and requires refs even when display-name metadata still matches
  the AST, covered by
  `typechecker::tests::check_program_with_symbols_rejects_extra_resolver_behavior_parent_refs`,
  `typechecker::tests::check_program_with_symbols_rejects_extra_resolver_behavior_impl_refs`
  and
  `typechecker::tests::check_program_with_symbols_rejects_extra_resolver_behavior_required_refs`.
- Resolver behavior-association diagnostics now use shared parent/impl/requires
  name/ref validation plumbing, reducing duplicate handoff code while preserving
  the existing explicit diagnostic labels and error codes.
- Resolver behavior-association collection now uses one resolver behavior-ref
  accessor for behavior parent refs, type impl refs, and type requires refs.
- Resolver behavior-association validation now derives display-name and typed
  ref expectations together, avoiding duplicate AST scans for the same edges.
- Resolver behavior-association expectation storage now uses one shared edge
  container for impl, requires, and parent inputs.
- Resolver behavior-association expectation storage now stores display names
  and typed refs as paired edges before deriving resolver comparison lists.
- Resolver behavior-association list validation now receives paired edge slices
  directly instead of wrapping the expected edge list.
- Resolver value-signature expectation building now uses one parameter pass for
  names, display types, and typed metadata.
- Resolver behavior-method expectation building now reuses the shared
  value-signature metadata path for names, display types, typed metadata, and
  returns.
- The docs truth gate now covers the quiet draft-PR CI trigger shape by
  rejecting `pull_request.synchronize`, requiring manual dispatch, and requiring
  the draft-PR guard on fmt, clippy, and test jobs.
- Resolver variant payload expectations now pass paired typed/display payload
  metadata directly to validation instead of wrapping it in a redundant
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
  routing for restored value metadata, covered by
  `typechecker::tests::callable_signature_insert_routes_function_and_method_keys`.
- Resolver-backed generic template restoration now shares function-vs-method
  routing for restored generic value metadata, covered by
  `typechecker::tests::generic_callable_template_mut_routes_function_and_method_keys`.
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
- Resolver value-parameter validation now shares one expected metadata bundle
  for counts, names, display types, and typed AST types.
- Resolver behavior-association list validation now shares one expected
  metadata bundle for display names and typed refs.
- Resolver behavior-association validation now uses one role/check mapping for
  parent, impl, and requires diagnostic metadata, covered by
  `typechecker::tests::behavior_ref_validation_maps_role_and_check_diagnostics`.
- Resolver behavior-association validation now uses the same role mapping for
  parent, impl, and requires resolver metadata selection, covered by
  `typechecker::tests::behavior_ref_actual_selects_role_metadata`.
- Resolver behavior-association contains/list validation now shares role-aware
  wrapper helpers for parent, impl, and requires diagnostics, covered by
  `typechecker::tests::behavior_ref_role_validation_emits_selected_contains_diagnostics`.
- Resolver struct-field validation now shares one expected metadata bundle
  for counts, display fields, and typed AST fields.
- Resolver behavior-method validation now shares one expected metadata bundle
  for display signatures and typed method metadata.
- Resolver variant-payload validation now shares one expected metadata bundle
  for counts, display payload types, and typed AST payloads.
- Resolver visibility validation now shares one diagnostic helper across
  module, import, local, type-like, variant, and value symbols.
- Resolver source validation now shares one diagnostic helper across module,
  import, and local symbols.
- Resolver local mutability validation now shares the same diagnostic helper
  shape used by the resolver metadata validation paths.
- Resolver extra-symbol validation now shares one diagnostic helper across
  declaration/import/module and local symbol checks.
- Resolver missing-symbol validation now shares one diagnostic helper across
  declaration/import/module/type/behavior/variant/value and local checks.
- Resolver absent-source metadata validation now reuses the shared resolver
  source diagnostic helper.
- Resolver stripped-import validation now reuses the shared resolver
  visibility and source diagnostic helpers.
- Resolver absent value-signature metadata validation now reuses the shared
  absent-metadata entry helper.
- Resolver absent value-signature detail validation now shares the same helper
  across module, import, local, type-like, and variant symbols.
- Resolver absent value-signature metadata validation now lets the validation
  bundle build its parameter and return metadata entries, covered by
  `typechecker::tests::value_signature_absence_validation_builds_entries`.
- Resolver absent type-parameter metadata validation now shares one helper
  across module, import, local, and variant symbols.
- Resolver absent type-parameter metadata validation now lets the validation
  bundle build its count/name/bounds metadata entries, covered by
  `typechecker::tests::type_parameter_absence_validation_builds_entries`.
- Resolver absent field metadata validation now shares one helper across
  module, import, local, enum, variant, behavior, and value symbols.
- Resolver absent field metadata validation now lets the validation bundle
  build its count/display/typed field metadata entries, covered by
  `typechecker::tests::field_absence_validation_builds_entries`.
- Resolver absent variant metadata validation now shares one helper across
  module, import, local, struct/type, behavior, and value symbols.
- Resolver absent variant metadata validation now lets the validation bundle
  build its names/owner/payload metadata entries, covered by
  `typechecker::tests::variant_absence_validation_builds_entries`.
- Resolver absent behavior-association metadata validation now shares one
  helper across module, import, local, variant, behavior, and value symbols.
- Resolver absent behavior-association metadata validation now lets the
  validation bundle build its impl/requires metadata entries, covered by
  `typechecker::tests::behavior_association_absence_validation_builds_entries`.
- Resolver absent behavior-declaration metadata validation now shares one
  helper across module, import, local, variant, and value symbols.
- Resolver absent behavior-declaration metadata validation now lets the
  validation bundle build its method/parent metadata entries, covered by
  `typechecker::tests::behavior_declaration_absence_validation_builds_entries`.
- Resolver absent mutability metadata validation now shares one helper across
  module, import, type-like, variant, and value symbols.

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
