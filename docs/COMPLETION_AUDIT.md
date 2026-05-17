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
- Nested generic enum specialization now has executable and generated-C
  evidence through `tests/zen/generic_nested_result_enum.zen`,
  `integration::test_generic_nested_result_enum`, and
  `integration::generic_specializations_do_not_emit_unspecialized_c_symbols`,
  proving `Result<Option<i32>, str>` does not leave undefined generated calls.
- Generic method specialization preserves concrete `Self` receiver context in
  call-site typing and specialized method bodies for generic struct and enum
  receivers, with executable and generated-C coverage in
  `tests/zen/generic_method_self.zen`, including receiver-based inference for
  `Self`-only generic method signatures and nested
  `Box<Option<i32>>` specialization dependency ordering.
- Generic method specialization with nested generic enum return types is covered
  by `tests/zen/generic_method_nested_result.zen`,
  `integration::test_generic_method_nested_result`, and
  `integration::generic_specializations_do_not_emit_unspecialized_c_symbols`,
  proving `Box<T>.wrap_result` can infer `T` from the concrete receiver and
  specialize `Result<Option<T>, str>` without unresolved generated C calls.
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
  Imported generic methods returning source-module generic enum dependencies
  now use receiver inference at the entry-module call and inside the source
  method body, covered by
  `tests/zen/multi_file_type_method_return_enum_dependency/main.zen` and
  `integration::test_multi_file_type_method_return_enum_dependency_imports`.
  Nested imported generic method return dependencies are also covered by
  `tests/zen/multi_file_type_method_nested_result_dependency/main.zen`,
  `integration::test_multi_file_type_method_nested_result_dependency_imports`,
  and `integration::generated_c_call_definition_scan_reports_missing_generated_calls`,
  proving `Result<Option<T>, str>` specialization does not leave undefined
  generated C calls.
- Imported generic `Result<T, E>` enum methods now cover multiple concrete
  instantiations in the importing module through
  `tests/zen/multi_file_generic_result_enum_multi_specialization/main.zen`,
  `integration::test_multi_file_generic_result_enum_multi_specialization_imports`,
  and `integration::generic_specializations_do_not_emit_unspecialized_c_symbols`,
  proving both `Result_unwrap_or_i32_str` and `Result_unwrap_or_bool_str`
  resolve to emitted definitions exactly once without unspecialized `Result_T`
  symbols.
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
  It also resolves stale AST declaration names through resolver symbols before
  validating collected type references and body type annotations, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_generic_method_name_for_body_type_refs`.
  When resolver value-signature metadata is incomplete and the collected
  signature/template is removed, body type-reference validation skips that
  declaration instead of falling back to stale AST generic parameters, covered
  by
  `typechecker::tests::collect_declarations_with_symbols_does_not_validate_stale_generic_function_body_refs_when_signature_incomplete`.
- Resolver-backed generic type-reference validation now has a focused
  resolver-backed traversal instead of interleaving resolver and AST-only
  validation in each declaration arm, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_function_signature_for_type_refs`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_method_signature_for_type_refs`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_name_for_body_type_refs`,
  and
  `typechecker::tests::collect_declarations_with_symbols_does_not_validate_stale_behavior_default_body_refs_when_methods_incomplete`.
- Resolver-backed function type-reference validation now runs through a
  dedicated restored-function helper, with focused coverage from
  `typechecker::tests::collect_declarations_with_symbols_does_not_validate_stale_generic_function_body_refs_when_signature_incomplete`
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_function_signature_for_type_refs`.
- Resolver-backed top-level method type-reference validation now runs through a
  dedicated restored-method helper, with focused coverage from
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_method_signature_for_type_refs`
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_generic_method_name_for_body_type_refs`.
- Resolver-backed `Type.impl` method type-reference validation now shares the
  restored-method helper, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_name_for_body_type_refs`.
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
- Resolver-backed non-behavior `Type.impl` metadata collection now takes the
  dispatcher-owned impl target and method list directly instead of re-matching
  the full declaration, with focused coverage from
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_method_name_metadata`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_target_name_metadata`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_target_and_name_metadata`.
- Impl-block declaration collection now dispatches the target and method list
  once for both AST collection and resolver-backed template stubs, shrinking
  another duplicate declaration walk while keeping the same `Type.impl`
  coverage above.
- Callable declaration collection now dispatches each function or top-level
  method once for both AST collection and resolver-backed template stubs,
  shrinking another duplicate function/method declaration walk while preserving
  generic callable template coverage.
- Resolver-backed callable type-reference validation now shares the same
  collected signature/body helper after each caller restores the resolver-owned
  function or method key, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_generic_function_name_for_body_type_refs`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_generic_method_name_for_body_type_refs`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_name_for_body_type_refs`.
- AST type declaration collection now validates struct/enum generic bounds in
  the same declaration dispatch that builds type metadata, removing the
  separate type-generic-bound pass while preserving generic type diagnostics.
- AST behavior declaration collection now queues behavior generic bounds while
  building behavior metadata, then validates them after all behavior names are
  collected without a separate behavior declaration walk, preserving
  declaration-order-independent behavior bound diagnostics.
- Behavior declaration collection now dispatches each behavior once and sends
  extracted signature fields into AST signatures or resolver-backed stubs,
  preserving existing behavior metadata coverage without whole-list handoff.
- AST behavior-extends validation now records explicit extends-validation
  tasks before replaying checks, preserving cycle and coherence validation
  order without passing the whole declaration list into the check loop.
- Struct field-default validation now dispatches struct declarations once and
  routes extracted fields through AST or resolver-restored default checks,
  preserving resolver field-name coverage without mode-specific declaration scans.
- AST callable type-reference validation now shares one signature/body helper
  across functions, top-level methods, and `Type.impl` methods while preserving
  each caller's existing return diagnostic span.
- `Self` type validation now shares one callable signature/body helper across
  functions, top-level methods, behavior default methods, and `Type.impl`
  methods while preserving each caller's existing `Self` allowance.
- `Self` type validation now also shares behavior-association type-argument
  validation across impl, requires, and extends declarations.
- Resolver-symbol validation now shares one callable local-symbol helper across
  functions, top-level methods, behavior default methods, and `Type.impl`
  methods while keeping declaration symbol checks at each call site.
- Resolver-symbol validation now shares generic behavior-association
  type-argument validation across impl, requires, and extends declarations.
- Generic type-reference validation now shares strict and unknown-tolerant
  type-argument list walking across recursive type refs, expression type args,
  and resolver-owned behavior association refs.
- Resolver validation replay now collects expected declaration symbols,
  expected local symbols, import-validation state, and behavior-association
  replay tasks in one declaration pass, covered by
  `typechecker::tests::resolver_validation_replay_tasks_collect_symbols_and_behavior_associations_together`.
  Existing focused coverage for expected symbols, behavior association lists,
  and stripped resolver imports remains in
  `typechecker::tests::resolver_expected_symbol_sets_collect_declarations_and_locals_together`,
  `typechecker::tests::resolver_behavior_association_list_tasks_collect_type_and_parent_edges_together`,
  and
  `typechecker::tests::check_program_with_symbols_validates_stripped_resolver_import_sources`.
  Behavior and non-behavior impl-block method expected-symbol collection also
  share one helper, covered by
  `typechecker::tests::expected_resolver_impl_method_symbols_collect_value_symbols_and_locals`.
  Callable parameter/body expected-local collection is shared, covered by
  `typechecker::tests::expected_resolver_callable_locals_collect_params_and_body`.
  Scoped expression local collection is shared for struct field defaults and
  top-level expressions, covered by
  `typechecker::tests::expected_resolver_scoped_expr_locals_collects_block_bindings`.
  The required resolver-symbol path uses the same scoped expression helper,
  covered by
  `typechecker::tests::check_program_with_symbols_requires_resolver_struct_field_default_locals`
  and
  `typechecker::tests::check_program_with_symbols_requires_resolver_top_level_expr_locals`.
  Closure parameters now reuse the same parameter-local helpers for expected
  and required resolver locals, preserving mutable parameter metadata.
  Closure body local collection is shared, covered by
  `typechecker::tests::expected_resolver_closure_locals_collects_params_and_body_bindings`,
  `typechecker::tests::check_program_with_symbols_requires_resolver_closure_locals`,
  and
  `typechecker::tests::check_program_with_symbols_validates_resolver_closure_parameter_mutability`.
  Child expression local collection is shared for loop, while, and conditional
  branch scopes, covered by
  `typechecker::tests::expected_resolver_child_expr_locals_collects_branch_bindings`.
  Match-arm pattern/body local collection is shared, covered by
  `typechecker::tests::expected_resolver_pattern_expr_locals_collects_pattern_and_body_bindings`.
  Pattern binding insertion is shared for identifier and struct shorthand
  bindings, covered by
  `typechecker::tests::expected_resolver_pattern_locals_collects_struct_shorthand_bindings`.
  Variable-declaration local binding and the mutable handoff predicate are
  shared between required and expected statement replay, covered by
  `typechecker::tests::expected_resolver_statement_locals_preserve_mutable_handoff`.
  Block expression/statement local collection is shared, covered by
  `typechecker::tests::expected_resolver_block_locals_collects_statement_and_final_expr_bindings`.
- Resolver-backed declaration metadata collection now records callable, type,
  and behavior metadata tasks in one declaration dispatch, then replays the
  same callable/type/behavior restoration order as before, shrinking duplicate
  resolver-backed declaration walks without changing restoration ordering.
- Declaration collection replay now records resolver semantic validation tasks
  beside AST collection and resolver metadata tasks in the same declaration
  pass, and resolver-backed semantic validation consumes that dedicated
  semantic task bundle instead of replaying from metadata tasks.
- Resolver replay declaration-validation tests now split semantic-bundle
  coverage into a focused submodule, keeping the anti-slop file-size guard
  below threshold while preserving the resolver replay coverage.
- CLI usage rendering now lives in `src/cli/usage.rs`, reducing the command
  dispatcher file size while preserving the existing usage text and command
  behavior.
- Build-graph JSON integration tests now split host-effect JSON coverage into
  `tests/integration/cli_build/build_graph_json_host_effects.rs`, keeping target
  metadata/dependency JSON tests separate from effect determinism tests.
- Generic specialization generated-C assertions now split multi-file import and
  dependency cases into
  `tests/integration/generic_specializations/multifile_generated_c.rs`,
  preserving generated-call coverage while reducing the root integration file.
- Check-command build graph validation now splits host-effect ordering and
  declared-effect tests into
  `tests/integration/cli_build/graph_validation_host_effects.rs`, leaving
  target source and library graph validation in the root graph-validation file.
- Resolver-backed behavior impl metadata now builds restored impl-block tasks
  once and reuses them for both impl method signature restoration and omitted
  default-method synthesis, preserving the signature-before-defaults ordering.
- Collected declaration semantic validation now records behavior impl and
  requires tasks in one declaration dispatch, then replays the same
  impl-before-requires validation order as before.
- Resolver-backed type behavior-impl refresh now uses explicit restored type
  tasks instead of a callback traversal for the final association restoration
  pass.
- Resolver-backed `Type.impl` method type-reference validation now has a
  dedicated impl-method helper, keeping impl-block method filtering out of the
  broader resolver-backed type-reference declaration scan.
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
- Method-key method-name parsing is also shared by resolver-backed behavior
  impl conformance and behavior impl signature collection, covered by
  `typechecker::tests::method_signature_key_helpers_share_receiver_parsing`,
  `typechecker::tests::impl_effective_method_name_prefers_resolver_then_ast_then_collected_signature`,
  and
  `typechecker::tests::resolver_backed_behavior_impl_method_signature_name_prefers_resolver_key`.
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
- Resolver symbol presence diagnostics now use centralized resolver-code
  bundles for missing declaration symbols, missing local symbols, extra
  declaration symbols, and extra local symbols, covered by
  `typechecker::tests::resolver_symbol_presence_validation_uses_resolver_codes`.
- Resolver source-mismatch diagnostics now use centralized resolver-code
  bundles for module, stripped import, import, and local symbols, covered by
  `typechecker::tests::source_validation_uses_resolver_codes`.
- Type-parameter resolver validation now adapts into shared count diagnostics
  through its validation helper, covered by
  `typechecker::tests::type_parameter_validation_builds_count_validation`.
- Resolver behavior-ref validation now separates role labels from per-check
  diagnostic codes, covered by
  `typechecker::tests::behavior_ref_validation_separates_role_labels_from_check_codes`.
- Resolver behavior-ref actual metadata selection now uses one role selector
  for parent, impl, and required refs, covered by
  `typechecker::tests::behavior_ref_actual_exposes_role_metadata_selection`.
- Resolver behavior-ref actual metadata now owns contains and full-list
  matching for display names and typed refs, covered by
  `typechecker::tests::behavior_ref_actual_matches_expected_edges`.
- Resolver behavior-ref owner restoration now separates exact behavior-key
  owner lookup from unique fallback owner lookup, covered by
  `typechecker::tests::resolver_behavior_ref_owner_prefers_exact_then_unique_fallbacks`.
- Resolver expected value parameter construction now pairs parameter names,
  display types, and typed AST types through one constructor, covered by
  `typechecker::tests::expected_parameter_builds_name_display_and_type_together`.
- Resolver expected return metadata construction now pairs default void
  handling, display return metadata, and typed AST return metadata through one
  constructor, covered by
  `typechecker::tests::expected_return_metadata_defaults_and_displays_together`.
- Resolver expected type-parameter construction now pairs generic bound display
  metadata and typed bound-ref metadata through one constructor, covered by
  `typechecker::tests::expected_type_parameter_builds_bound_display_and_ref_together`.
- Resolver expected struct-field construction now pairs field display metadata
  and typed field metadata through one constructor, covered by
  `typechecker::tests::expected_field_builds_display_and_type_together`.
- Resolver expected enum-variant payload construction now pairs optional
  payload display metadata and typed payload metadata through one constructor,
  covered by
  `typechecker::tests::expected_variant_payload_builds_display_and_type_together`.
- Resolver expected behavior-method construction now pairs display method
  signatures and typed method metadata through one constructor, covered by
  `typechecker::tests::expected_behavior_method_builds_signature_and_metadata_together`.
- Resolver expected value-signature construction now gathers parameter, return,
  and type-parameter expectations through one constructor, covered by
  `typechecker::tests::expected_value_signature_builds_components_together`.
- Resolver expected value-symbol construction now pairs value signature
  expectations with visibility through one constructor, covered by
  `typechecker::tests::expected_value_symbol_builds_signature_and_visibility_together`.
- Resolver expected type-like symbol construction now pairs generic
  type-parameter expectations with optional visibility through one constructor,
  covered by
  `typechecker::tests::expected_type_like_symbol_builds_type_params_and_visibility_together`.
- Resolver expected behavior-symbol construction now pairs type-like
  expectations and behavior-method expectations through one constructor,
  covered by
  `typechecker::tests::expected_behavior_symbol_builds_type_like_and_methods_together`.
- Resolver expected struct-symbol construction now pairs type-like expectations
  and field expectations through one constructor, covered by
  `typechecker::tests::expected_struct_symbol_builds_type_like_and_fields_together`.
- Resolver expected enum-symbol construction now pairs type-like expectations
  and variant-name expectations through one constructor, covered by
  `typechecker::tests::expected_enum_symbol_builds_type_like_and_variants_together`.
- Resolver expected variant-symbol construction now pairs owner, visibility,
  and payload expectations through one constructor, covered by
  `typechecker::tests::expected_variant_symbol_builds_owner_visibility_and_payload_together`.
- Resolver expected import-symbol construction now pairs import source
  expectations and default visibility through one constructor, covered by
  `typechecker::tests::expected_import_symbol_builds_source_and_visibility_together`.
- Resolver expected module-symbol construction now pairs module name, absent
  source, and default visibility through one constructor, covered by
  `typechecker::tests::expected_module_symbol_builds_name_source_and_visibility_together`.
- Resolver expected local-symbol construction now pairs local scope, mutability,
  absent source, and default visibility through one constructor, covered by
  `typechecker::tests::expected_local_symbol_builds_scope_mutability_source_and_visibility_together`.
- Resolver expected behavior association construction now pairs display names
  and typed refs through one constructor, covered by
  `typechecker::tests::expected_behavior_edge_builds_display_and_metadata_together`.
- Resolver expected behavior-association aggregation now pairs impl and
  required edge groups through one constructor, covered by
  `typechecker::tests::expected_behavior_associations_build_impl_and_required_edges_together`.
- Resolver expected behavior-parent aggregation now pairs `.extends` owners
  and typed parent edges through one constructor, covered by
  `typechecker::tests::expected_behavior_edges_build_parent_edges_from_extends_together`.
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
  Malformed nested generic type annotations inside explicit call type
  arguments stop before dependent call-signature checks.
  Generic method explicit type arguments also reject bare generic type
  annotations with missing type arguments.
  Generic function and method type-argument inference conflicts now report
  direct diagnostics for the conflicting parameter and concrete types,
  including receiver-derived generic method type arguments that conflict with
  later call arguments.
  Generic inference also walks function, array, and raw-pointer parameter
  shapes, so nested type parameters inside compound arguments can produce
  direct conflict diagnostics. Generic method inference conflicts now have
  matching compound-shape coverage for function, array, raw-pointer, and slice
  parameter types in `tests/generic_diagnostics.rs`.
  Resolver rejects duplicate generic type-parameter names across value, type,
  and behavior declarations, covered by
  `resolver_phase2::resolver_rejects_duplicate_type_parameter_names`.
  Generic behavior bound failures are covered across direct function, method,
  generic-receiver method, and UFC-style function call paths, and bound
  failures skip dependent specialization-body diagnostics.
- Typechecker setup accepts resolver `SymbolTable` through
  `check_program_with_symbols`.
- Resolver behavior impl method symbols carry complete value-signature metadata,
  and typechecker setup rejects impl-method signature drift before behavior impl
  body collection. Function-typed impl-method parameters and returns are
  included in that handoff coverage.
- Behavior impl conformance reads the collected `Type.method` signature,
  including resolver-restored impl-method metadata, so stale AST-only method
  signatures cannot produce false impl diagnostics.
- Generic behavior impl method template restoration uses the shared resolver
  callable key repair path, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata`
  and
  `typechecker::tests::collect_declarations_with_symbols_clears_stale_behavior_impl_generic_method_template_after_key_restore`,
  so stale AST target/name keys cannot survive restored or incomplete resolver
  value-signature metadata.
- Resolver-backed behavior impl conformance also restores impl method names
  from resolver-owned value symbols when AST-only impl method names are stale,
  without masking real extra impl methods that lack resolver-owned required
  method symbols.
  Direct coverage for stale AST names hiding real resolver-owned extra methods
  is provided by
  `typechecker::tests::collect_declarations_with_symbols_does_not_let_stale_ast_name_hide_extra_impl_method`.
  Stale AST impl method parameter names and ordering are covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_impl_method_parameter_names_for_impl_checks`
  and
  `typechecker::tests::collect_declarations_with_symbols_ignores_stale_impl_method_parameter_order_for_impl_checks`.
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
- Resolver records mutable closure parameter locals and typechecker
  resolver-backed validation rejects closure parameter mutability drift, covered
  by `resolver_phase2::resolver_records_mutable_closure_parameter_locals` and
  `typechecker::tests::check_program_with_symbols_validates_resolver_closure_parameter_mutability`.
- Resolver type symbols carry behavior impl and `.requires` association
  metadata, and typechecker setup rejects missing or extra association metadata
  before collecting behavior impls/requires from the AST. Specialized behavior
  references such as `Json<str>` are included in this resolver handoff
  validation.
- Resolver-backed `.requires` conformance checks read validated resolver
  required-behavior refs, so stale AST-only required behavior type arguments
  cannot produce false missing-impl diagnostics.
  Restored missing-impl diagnostics for stale target type names, behavior names,
  and behavior type arguments are covered by
  `typechecker::tests::collect_declarations_with_symbols_reports_resolver_restored_required_target_and_name`.
- Resolver-backed `.implements` conformance checks read validated resolver
  behavior impl refs before method conformance, so stale AST-only impl behavior
  type arguments cannot produce false method signature diagnostics.
  Restored missing-method diagnostics for stale target type names, behavior
  names, and behavior type arguments are covered by
  `typechecker::tests::collect_declarations_with_symbols_reports_resolver_restored_impl_target_and_name`.
- Resolver-backed `.implements` and `.requires` conformance also falls back to
  declaration-order resolver refs when AST-only behavior names are stale, so
  validated resolver behavior associations cannot be shadowed by stale AST names
  during semantic checks.
  Explicit generic `.implements` target, behavior name, and type-argument
  restoration together are covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_impl_target_and_name_metadata`.
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
- Distinct generic behavior specializations implemented by the same concrete
  type dispatch through behavior-specialized impl method symbols, covered by
  `tests/zen/behavior_distinct_generic_specialization_dispatch.zen` and
  generated-C assertions for `Point_encode__Json_str` and
  `Point_encode__Json_i32`.
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
  Resolver-restored generic impl refs also drive overlap diagnostics when
  AST-only impl type arguments are stale, covered by
  `typechecker::tests::collect_declarations_with_symbols_reports_overlap_from_restored_impl_type_args`.
  Restored generic impl refs also prevent false duplicate diagnostics when
  AST-only impl type arguments collapse distinct specializations, covered by
  `typechecker::tests::collect_declarations_with_symbols_avoids_false_duplicate_from_restored_impl_type_args`.
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
- Resolver-backed generic function and method template collection now seeds
  body-only template stubs before resolver metadata restoration, preserving
  positional mutability and spans without carrying AST-only generic names,
  parameter types, or return annotations, covered by
  `typechecker::tests::resolver_backed_callable_template_collection_defers_signature_metadata_to_resolver`.
- AST `Type.impl` method signature collection owns generic-bound validation
  directly after resolver-backed impl collection split into its own template
  pass, covered by `typechecker::tests::type_impl_method_collection` and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_impl_method_bounds_for_validation`.
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
- Resolver-backed behavior default synthesis now uses a named skip helper for
  incomplete resolver impl-ref handoff, covered by
  `typechecker::tests::behavior_default_synthesis_skip_requires_resolver_collection_and_missing_impl_ref`.
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
  Resolver-restored generic template parameter names no longer steal mutability
  from stale AST parameters with matching names in different positions, covered
  by
  `typechecker::tests::collect_declarations_with_symbols_ignores_stale_generic_template_param_names_for_mutability`.
- Top-level generic method template return presence and parameter counts are
  restored from resolver metadata, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_generic_method_template_return_presence`
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_generic_method_template_parameter_count`.
  Resolver-restored top-level generic method parameter names also preserve
  positional mutability and avoid stale same-name AST parameter matches,
  covered by
  `typechecker::tests::collect_declarations_with_symbols_preserves_generic_method_template_param_mutability_by_position`
  and
  `typechecker::tests::collect_declarations_with_symbols_ignores_stale_generic_method_template_param_names_for_mutability`.
- Resolver-backed generic `Type.impl` method template collection preserves
  function-typed parameter/return metadata and behavior-bound metadata from
  resolver symbols instead of stale AST-only generic signatures, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_metadata`.
  Resolver-restored return presence and parameter counts are covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_return_presence`
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_parameter_count`.
  Resolver-restored generic impl method parameter names preserve positional
  mutability and avoid stale same-name AST parameter matches, covered by
  `typechecker::tests::collect_declarations_with_symbols_preserves_type_impl_generic_template_param_mutability_by_position`
  and
  `typechecker::tests::collect_declarations_with_symbols_ignores_stale_type_impl_generic_template_param_names_for_mutability`.
- Resolver-backed behavior method collection rebuilds behavior parameters from
  resolver-owned parameter names and types, so stale AST-only missing or extra
  parameters cannot distort impl conformance checks.
  Stale AST behavior method parameter names and parameter ordering are covered
  by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_method_parameter_names`
  and
  `typechecker::tests::collect_declarations_with_symbols_ignores_stale_behavior_method_parameter_order`.
- Resolver-backed behavior method collection also walks resolver-owned behavior
  method metadata in resolver order, so stale AST-only missing behavior methods
  cannot drop required methods from impl conformance checks.
- Resolver-backed behavior collection now seeds only behavior method/default
  stubs before resolver metadata restoration, so generic names and bounds come
  from resolver symbols instead of the initial AST collection pass, covered by
  `typechecker::tests::resolver_backed_behavior_collection_defers_generic_metadata_to_resolver`.
- When resolver behavior-method metadata is incomplete and behavior collection
  is removed, default-body type-reference validation skips that behavior instead
  of falling back to stale AST generic parameters, covered by
  `typechecker::tests::collect_declarations_with_symbols_does_not_validate_stale_behavior_default_body_refs_when_methods_incomplete`.
- Resolver-backed behavior type-reference and default-body validation now runs
  through a dedicated restored-behavior helper, with focused coverage from
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_type_params_for_type_refs`
  and
  `typechecker::tests::collect_declarations_with_symbols_does_not_validate_stale_behavior_default_body_refs_when_methods_incomplete`.
- Resolver-backed struct and enum collection uses typed resolver generic bound
  refs, so generic type templates no longer retain stale AST-only behavior
  bounds after resolver validation.
- Resolver-backed struct type-reference and field-default expression validation
  now runs through a dedicated restored-struct helper, with focused coverage
  from
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_metadata_for_type_refs`
  and
  `typechecker::tests::collect_declarations_with_symbols_does_not_validate_stale_struct_field_default_refs_when_fields_incomplete`.
- Resolver-backed enum type-reference validation now runs through a dedicated
  restored-enum helper, with focused coverage from
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_metadata_for_type_refs`
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_enum_payload_metadata`.
- Struct field default expressions now participate in generic type-reference
  validation, so local annotations inside defaults cannot hide unknown type
  symbols, covered by
  `typechecker::tests::check_program_rejects_unknown_type_references_in_struct_field_defaults`.
  Non-generic struct field defaults are also checked against their declared
  field type, covered by
  `typechecker::tests::check_program_rejects_struct_field_default_type_mismatch`.
  Non-generic struct literals can omit defaulted fields and receive the typed
  default expression in declaration order, covered by
  `typechecker::tests::struct_literal_uses_default_for_omitted_field`.
  Generic struct literals also install concrete type substitutions while
  checking omitted default expressions, covered by
  `typechecker::tests::generic_struct_literal_uses_substituted_default_for_omitted_field`.
  When resolver field metadata is incomplete and struct collection is removed,
  default-expression validation skips that struct instead of falling back to
  stale AST generic parameters, covered by
  `typechecker::tests::collect_declarations_with_symbols_does_not_validate_stale_struct_field_default_refs_when_fields_incomplete`.
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
- Resolver behavior-ref selection and behavior impl required-method restoration
  now share one exact-match-then-front queue selector, covered by
  `typechecker::tests::named_queue_selection_prefers_exact_then_front`,
  `typechecker::tests::resolver_behavior_ref_queue_selection_prefers_exact_then_front`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_impl_method_name_metadata_for_impl_checks`.
- Resolver behavior method metadata restoration now uses the same named queue
  selection family while preserving front AST methods that later resolver
  method metadata still needs, covered by
  `typechecker::tests::named_queue_selection_can_preserve_front_for_future_match`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_method_name_metadata`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_method_count`.
- Behavior impl conformance now resolves effective method names through a
  dedicated helper that shares resolver-owned name, AST-name, and collected
  signature fallback selection, covered by
  `typechecker::tests::impl_effective_method_name_prefers_resolver_then_ast_then_collected_signature`,
  `typechecker::tests::resolver_backed_impl_method_key_requires_resolver_collection`,
  `typechecker::tests::resolver_backed_behavior_impl_method_signature_name_prefers_resolver_key`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_impl_method_signature_target_and_name_metadata`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_impl_method_name_metadata_for_impl_checks`,
  and
  `typechecker::tests::collect_declarations_with_symbols_does_not_let_stale_ast_name_hide_extra_impl_method`.
- Resolver-backed behavior impl conformance and default-method suppression now
  share one collected method-signature lookup helper, covered by
  `typechecker::tests::resolver_backed_method_signature_requires_resolver_collection`,
  `typechecker::tests::impl_effective_method_name_prefers_resolver_then_ast_then_collected_signature`,
  and
  `typechecker::tests::collect_declarations_with_symbols_skips_default_when_resolver_restores_impl_method_name`.
- Impl method collection, resolver-backed impl restoration, default seeding,
  and resolver-backed method lookup now share one type-qualified method key
  helper, covered by
  `typechecker::tests::method_key_formats_type_qualified_method_name`,
  `typechecker::tests::resolver_backed_method_signature_requires_resolver_collection`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_impl_method_name_metadata_for_impl_checks`.
- Resolver-backed method signature collection and generic type-reference
  validation now use the same type-qualified method key helper, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_method_signature_for_type_refs`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_name_for_body_type_refs`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_generic_method_name_for_body_type_refs`.
- Resolver symbol validation for top-level and impl method signatures now uses
  the shared type-qualified method key helper, covered by
  `typechecker::tests::check_program_with_symbols_validates_resolver_method_signature`,
  `typechecker::tests::check_program_with_symbols_validates_resolver_impl_method_signature`,
  and
  `typechecker::tests::method_key_formats_type_qualified_method_name`.
- Behavior impl conformance now uses the shared type-qualified method key
  helper before resolver-owned method name restoration, covered by
  `typechecker::tests::resolver_backed_impl_method_key_requires_resolver_collection`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_impl_method_name_metadata_for_impl_checks`,
  `typechecker::tests::collect_declarations_with_symbols_does_not_let_stale_ast_name_hide_extra_impl_method`,
  and
  `typechecker::tests::method_key_formats_type_qualified_method_name`.
- AST, resolver-backed, graph-import, dependency, and typed body method-key
  construction now route through the same type-qualified method key helper,
  covered by
  `typechecker::tests::method_signature_key_helpers_share_receiver_parsing`,
  `typechecker::tests::method_key_formats_type_qualified_method_name`, and
  `typechecker::tests::callable_signature_insert_routes_function_and_method_keys`.
- Expression method lookup for module fallbacks, concrete receivers, and
  generic receiver bases now also uses that shared method-key helper, covered
  by `typechecker::tests::generic_method_collection`,
  `generic_diagnostics::generic_method_inference_conflict_from_receiver_is_error`,
  and the `integration::test_generic_method*` fixtures.
- Resolver value-symbol definition for top-level and impl methods now also
  routes through a single type-qualified method key helper, covered by
  `resolver::tests::resolver_method_key_formats_type_qualified_method_name`,
  `resolver_phase2::resolver_records_method_signatures_as_value_symbols`, and
  `resolver_phase2::resolver_accepts_non_behavior_impl_blocks_as_method_symbols`.
- Resolver-backed declaration collection now has named passes for resolver
  declaration metadata refresh, behavior impl metadata refresh, semantic
  validation, and final impl association refresh, reducing the mixed
  declaration collection surface that later Phase 2 slices still need to shrink.
  Callable resolver declaration metadata now has a focused traversal for
  functions, top-level methods, and non-behavior impl methods, with
  function/method arms calling the shared signature restoration helpers
  directly, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_method_signature_for_type_refs`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_impl_method_name_metadata`,
  and
  `typechecker::tests::resolver_declaration_metadata_skips_behavior_impl_methods_until_behavior_impl_pass`.
  Type resolver declaration metadata now has a focused traversal for structs
  and enums, with shared resolver-owned type-name and behavior-ref restoration
  before type-specific field or variant collection, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_struct_field_metadata`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_struct_name_metadata`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_enum_payload_metadata`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_enum_name_metadata`.
  Behavior resolver declaration metadata now has a focused traversal for
  behavior declarations that calls the shared behavior metadata collector
  directly, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_method_metadata`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_name_metadata`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_method_name_metadata`.
  Behavior impl method signatures are skipped by the generic declaration
  metadata refresh and owned by the behavior impl metadata pass, covered by
  `typechecker::tests::resolver_declaration_metadata_skips_behavior_impl_methods_until_behavior_impl_pass`.
  AST behavior declaration seeding, behavior generic-bound validation, and
  AST-only behavior inheritance validation now also have named helper passes.
  AST behavior signature seeding and resolver-backed behavior stub seeding are
  separate helper passes, covered by
  `typechecker::tests::behavior_declaration_collection`,
  `typechecker::tests::resolver_backed_behavior_collection_defers_generic_metadata_to_resolver`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_method_metadata`.
  Behavior inheritance validation now dispatches through a shared self-type
  context pass and an AST-only extends/coherence helper, covered by
  `typechecker::tests::behavior_extends_validation_tasks_collect_parent_refs`,
  `typechecker::tests::behavior_extends_requires_parent_methods`,
  `typechecker::tests::behavior_extends_cycle_is_error`,
  `typechecker::tests::behavior_extends_conflicting_method_signature_is_error`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_parent_metadata`.
  Behavior declaration collection now dispatches to AST signature seeding plus
  behavior generic-bound validation, or resolver-backed stub seeding, avoiding
  duplicate AST-only diagnostics from the remaining collection loop, covered by
  `typechecker::tests::behavior_declaration_collection`,
  `typechecker::tests::behavior_generic_bound_accepts_later_behavior_declaration`,
  `typechecker::tests::resolver_backed_behavior_collection_defers_generic_metadata_to_resolver`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_bounds_for_validation`.
  AST struct/enum generic-bound validation and type declaration seeding now
  also have named helper passes.
  Type declaration collection now dispatches to that AST-only path instead of
  invoking guarded type helpers during resolver-backed collection, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_metadata_for_type_refs`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_type_bounds_for_validation`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_struct_name_metadata`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_enum_name_metadata`.
  AST callable generic-bound validation, callable signature seeding, and
  resolver-backed callable template seeding now also have named helper passes.
  Callable collection now dispatches to exactly one of AST generic-bound
  validation plus signature seeding, or resolver-backed template seeding,
  instead of invoking guarded AST-only passes during resolver-backed
  collection, covered by `typechecker::tests::generic_function_collection`,
  `typechecker::tests::generic_method_collection`,
  `typechecker::tests::resolver_backed_callable_template_collection_defers_signature_metadata_to_resolver`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_generic_function_template_name_metadata`.
  AST impl method/default seeding and resolver-backed impl template seeding now
  also have named helper passes.
  Impl-block collection now dispatches to exactly one of those passes instead
  of invoking both and relying on per-pass resolver-backed guards, covered by
  `typechecker::tests::type_impl_method_collection`,
  `typechecker::tests::behavior_impl_can_omit_default_method`,
  `typechecker::tests::resolver_backed_callable_template_collection_defers_signature_metadata_to_resolver`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata`.
  AST import seeding now has a named helper pass, removing the residual mixed
  declaration collection loop.
  Test-facing resolver replay task views now use focused slice collectors
  instead of collecting the full resolver declaration metadata bundle before
  discarding unrelated task lists, covered by
  `cargo test resolver_type_declaration_metadata_tasks_collect_only_type_work`,
  `cargo test resolver_callable_declaration_metadata_tasks_collect_callable_work`,
  `cargo test resolver_behavior_impl_block_declaration_tasks_collect_only_behavior_impls`,
  and `cargo test resolver_type_reference_validation_tasks_collect_only_type_reference_work`.
  Resolver-backed function and method signature restoration now shares one
  callable key repair and generic-template rekey helper.
  Resolver-backed semantic validation and final type impl association refresh
  now each have focused helper boundaries matching those named passes.
  Callable signatures, type declarations, and behavior declarations now route
  through focused declaration metadata helpers within that resolver refresh
  pass.
  Behavior declaration metadata now has the same focused helper boundary as
  callable and type declaration metadata.
- Resolver-backed `.requires` conformance uses restored required-behavior refs
  together with inherited child behavior impl satisfaction, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_restored_requires_ref_for_inherited_impl`.
  Distinct restored generic `.requires` refs remain satisfied when stale AST
  type arguments collapse the requirements, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_distinct_restored_requires_type_args`.
- Resolver-backed behavior inheritance checks validate restored resolver parent
  refs before cycle and method-coherence checks, so stale AST-only parent names
  or type arguments cannot leak false extends diagnostics.
  Cycle diagnostics from restored parent refs are covered by
  `typechecker::tests::collect_declarations_with_symbols_reports_cycle_from_restored_parent_refs`.
  Restored generic parent refs prevent false duplicate inheritance diagnostics
  when AST-only parent type arguments collapse distinct parent specializations,
  covered by
  `typechecker::tests::collect_declarations_with_symbols_avoids_false_duplicate_from_restored_parent_type_args`.
  Inherited method-coherence diagnostics from restored generic parent refs are
  covered by
  `typechecker::tests::collect_declarations_with_symbols_reports_conflict_from_restored_parent_type_args`.
  Inherited missing-method diagnostics from restored parent refs are covered by
  `typechecker::tests::collect_declarations_with_symbols_reports_resolver_restored_behavior_parent_metadata`.
  Inherited behavior default synthesis from restored parent refs is covered by
  `typechecker::tests::collect_declarations_with_symbols_synthesizes_defaults_from_restored_behavior_parent`.
  Restored generic parent type arguments in inherited default synthesis are
  covered by
  `typechecker::tests::collect_declarations_with_symbols_synthesizes_generic_defaults_from_restored_parent_args`.
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
- Omitted behavior default synthesis uses resolver-restored behavior method
  names, so stale AST-only default method names do not synthesize stale method
  keys, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_default_method_name_metadata`.
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
- Resolver-backed struct field default validation now uses a dedicated
  resolver-restored field-default helper, with focused coverage from
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_struct_name_metadata`,
  `typechecker::tests::collect_declarations_with_symbols_does_not_validate_stale_struct_field_default_refs_when_fields_incomplete`,
  and
  `typechecker::tests::collect_declarations_with_symbols_clears_stale_struct_fields_after_name_restore`.
- AST-only and resolver-backed struct field default declaration traversal now
  have separate helper passes, covered by
  `typechecker::tests::struct_literal_uses_default_for_omitted_field`,
  `typechecker::tests::generic_struct_literal_uses_substituted_default_for_omitted_field`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_struct_field_names_for_defaults`,
  and
  `typechecker::tests::collect_declarations_with_symbols_does_not_validate_stale_struct_field_default_refs_when_fields_incomplete`.
- Resolver-backed struct field defaults are stored and validated under
  resolver-owned field names by position, so stale AST-only field names cannot
  skip default type checking, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_struct_field_names_for_defaults`.
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
  accessor for behavior parent refs, type impl refs, and type requires refs;
  resolver-backed impl/requires semantic checks also pop restored refs through
  one role-selected helper, covered by
  `typechecker::tests::resolver_behavior_ref_for_selects_impl_and_required_queues_by_role`.
- Resolver-backed callable template and behavior method collection now share
  resolver-owned parameter restoration while preserving AST mutability and
  spans, covered by
  `typechecker::tests::resolver_params_from_metadata_preserves_ast_param_shape`.
- Resolver-backed callable template and behavior method collection now share
  resolver return-type restoration for `void` versus annotated returns, covered
  by
  `typechecker::tests::resolver_optional_return_type_maps_void_to_missing_annotation`.
- Resolver-backed enum collection now uses one helper to restore resolver-owned
  variant names and owner-scoped typed payload metadata, covered by
  `typechecker::tests::resolver_enum_variants_from_metadata_uses_owner_scoped_payloads`.
- Resolver-backed struct collection now restores resolver-owned field names,
  typed field metadata, and field defaults through one helper, covered by
  `typechecker::tests::resolver_struct_fields_from_metadata_restores_field_names_and_defaults`.
- Resolver-backed behavior collection now restores resolver-owned method lists
  and AST default bodies through one metadata helper, covered by
  `typechecker::tests::resolver_behavior_methods_from_metadata_preserves_defaults_by_resolver_order`.
- Resolver-backed behavior parent collection now restores parent refs and
  computed behavior keys through one metadata helper, covered by
  `typechecker::tests::behavior_parent_refs_from_metadata_restores_keys_and_type_args`.
- Resolver-backed type implementation collection now restores impl association
  keys from resolver behavior metadata through one helper, covered by
  `typechecker::tests::behavior_impl_refs_from_metadata_restores_type_and_behavior_keys`.
- Resolver behavior-association validation now derives display-name and typed
  ref expectations together, avoiding duplicate AST scans for the same edges.
- Resolver behavior-association expectation storage now uses one shared edge
  container for impl, requires, and parent inputs.
- Resolver behavior-association expectation storage now stores display names
  and typed refs as paired edges before deriving resolver comparison lists.
- Resolver behavior-association list validation now receives paired edge slices
  directly instead of wrapping the expected edge list.
- Resolver-backed `.requires` semantic validation now restores the required
  type target through a dedicated declaration helper, with focused coverage
  from
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_required_metadata`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_required_target_metadata`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_required_target_and_name_metadata`,
  and
  `typechecker::tests::collect_declarations_with_symbols_reports_resolver_restored_required_target_and_name`.
- Resolver-backed behavior-impl semantic validation now restores the impl type
  target through a dedicated declaration helper, with focused coverage from
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_impl_method_metadata_for_impl_checks`,
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_impl_method_name_metadata_for_impl_checks`,
  and
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata`.
- Resolver value-signature expectation building now uses one parameter pass for
  names, display types, and typed metadata.
- Resolver behavior-method expectation building now reuses the shared
  value-signature metadata path for names, display types, typed metadata, and
  returns.
- Resolver-backed callable signature collection now reads resolver parameter
  names, parameter types, and return types through one complete-signature
  helper, covered by
  `typechecker::tests::resolver_callable_signature_metadata_requires_complete_signature`.
- Resolver-backed struct field collection now reads resolver field metadata
  through one dedicated helper before restoring fields and defaults, covered by
  `typechecker::tests::resolver_struct_field_metadata_requires_field_types`.
- Resolver-backed enum variant collection now reads resolver variant-name
  metadata through one dedicated helper before restoring owner-scoped payloads,
  covered by
  `typechecker::tests::resolver_enum_variant_name_metadata_requires_variant_names`.
- Resolver-backed behavior method collection now reads resolver method metadata
  through one dedicated helper before restoring method signatures and defaults,
  covered by
  `typechecker::tests::resolver_behavior_method_metadata_requires_method_types`.
- Resolver-backed declaration info now reads resolver type-parameter names and
  typed bound refs through one shared metadata helper, covered by
  `typechecker::tests::resolver_type_parameter_metadata_requires_names_and_bound_refs`.
- Resolver-backed generic template refresh now uses the same complete
  type-parameter metadata as callable info, covered by
  `typechecker::tests::collect_declarations_with_symbols_clears_generic_function_template_type_params_when_resolver_bounds_missing`
  and
  `typechecker::tests::collect_declarations_with_symbols_clears_generic_method_template_type_params_when_resolver_bounds_missing`.
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
- Resolver type-parameter validation now carries name, display-bound, and
  typed-bound-ref message formatting through the validation bundle, covered by
  `typechecker::tests::type_parameter_validation_formats_messages`.
- Resolver count validation now carries count diagnostic message formatting
  through the shared count validation bundle, covered by
  `typechecker::tests::count_validation_formats_message`.
- Resolver value-parameter validation now shares one expected metadata bundle
  for counts, names, display types, and typed AST types.
- Resolver value-parameter validation now carries name, display-type, and
  typed-type message formatting through a validation bundle, covered by
  `typechecker::tests::value_parameter_validation_formats_messages`.
- Resolver value return-type validation now carries display and typed return
  message formatting through a validation bundle, covered by
  `typechecker::tests::return_validation_formats_messages`.
- Resolver value return-type validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::return_validation_uses_resolver_codes`.
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
- Resolver behavior-association diagnostics now carry contains/list name/ref
  message formatting through the behavior-ref validation bundle, covered by
  `typechecker::tests::behavior_ref_validation_maps_role_and_check_diagnostics`.
- Resolver struct-field validation now shares one expected metadata bundle
  for counts, display fields, and typed AST fields.
- Resolver struct-field validation now carries display-field and typed-field
  message formatting through a validation bundle, covered by
  `typechecker::tests::field_validation_formats_messages`.
- Resolver struct-field validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::field_validation_uses_resolver_codes`.
- Resolver behavior-method validation now shares one expected metadata bundle
  for display signatures and typed method metadata.
- Resolver behavior-method validation now carries display-method and
  typed-method message formatting through a validation bundle, covered by
  `typechecker::tests::behavior_method_validation_formats_messages`.
- Resolver behavior-method validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::behavior_method_validation_uses_resolver_codes`.
- Resolver variant-payload validation now shares one expected metadata bundle
  for counts, display payload types, and typed AST payloads.
- Resolver variant-payload validation now carries display-payload and
  typed-payload message formatting through a validation bundle, covered by
  `typechecker::tests::variant_payload_validation_formats_messages`.
- Resolver variant-payload validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::variant_payload_validation_uses_resolver_codes`.
- Resolver variant owner-name validation now carries its diagnostic code and
  message formatting through a validation bundle, covered by
  `typechecker::tests::variant_owner_validation_formats_message`.
- Resolver variant owner-name validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::variant_owner_validation_uses_resolver_code`.
- Resolver variant-name validation now carries its diagnostic code and message
  formatting through a validation bundle, covered by
  `typechecker::tests::variant_name_validation_formats_message`.
- Resolver variant-name validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::variant_name_validation_uses_resolver_code`.
- Resolver visibility validation now shares one diagnostic helper across
  module, import, local, type-like, variant, and value symbols.
- Resolver visibility validation now carries its diagnostic code and
  actual/expected display formatting through a validation bundle, covered by
  `typechecker::tests::visibility_validation_formats_actual_and_expected`.
- Resolver visibility validation now owns its full diagnostic message
  formatting through that bundle, keeping it aligned with source/count
  validation helpers.
- Resolver module visibility validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::visibility_validation_uses_module_resolver_code`.
- Resolver import visibility validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::visibility_validation_uses_import_resolver_code`.
- Resolver type-like visibility validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::visibility_validation_uses_type_like_resolver_code`.
- Resolver variant visibility validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::visibility_validation_uses_variant_resolver_code`.
- Resolver value visibility validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::visibility_validation_uses_value_resolver_code`.
- Resolver local visibility validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::visibility_validation_uses_local_resolver_code`.
- Resolver source validation now shares one diagnostic helper across module,
  import, and local symbols.
- Resolver source validation now carries source diagnostic message formatting
  through the shared source validation bundle, covered by
  `typechecker::tests::source_validation_formats_message`.
- Resolver type-like source absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::source_absence_validation_uses_type_like_resolver_code`.
- Resolver variant source absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::source_absence_validation_uses_variant_resolver_code`.
- Resolver value source absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::source_absence_validation_uses_value_resolver_code`.
- Resolver local mutability validation now shares the same diagnostic helper
  shape used by the resolver metadata validation paths.
- Resolver module mutability absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::mutability_absence_validation_uses_module_resolver_code`.
- Resolver import mutability absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::mutability_absence_validation_uses_import_resolver_code`.
- Resolver type-like mutability absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::mutability_absence_validation_uses_type_like_resolver_code`.
- Resolver variant mutability absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::mutability_absence_validation_uses_variant_resolver_code`.
- Resolver value mutability absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::mutability_absence_validation_uses_value_resolver_code`.
- Resolver local mutability validation now carries its diagnostic code and
  actual/expected display formatting through a validation bundle, covered by
  `typechecker::tests::mutability_validation_formats_actual_and_expected`.
- Resolver local mutability validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::mutability_validation_uses_resolver_code`.
- Resolver local mutability validation now owns its full diagnostic message
  formatting through that bundle, keeping it aligned with the
  source/count/visibility validation helpers.
- Resolver extra-symbol validation now shares one diagnostic helper across
  declaration/import/module and local symbol checks.
- Resolver missing-symbol validation now shares one diagnostic helper across
  declaration/import/module/type/behavior/variant/value and local checks.
- Resolver extra- and missing-symbol validation now push diagnostics through
  one shared presence helper, covered by
  `typechecker::tests::resolver_symbol_presence_validation_pushes_diagnostic`.
- Resolver extra- and missing-symbol validation now share one presence
  validation bundle for diagnostic codes and message formatting, covered by
  `typechecker::tests::resolver_symbol_presence_validation_formats_messages`.
- Resolver absent-source metadata validation now reuses the shared resolver
  source diagnostic helper.
- Resolver absent-source metadata validation now uses a validation bundle to
  build the source diagnostic configuration, covered by
  `typechecker::tests::source_absence_validation_builds_source_validation`.
- Resolver stripped-import validation now reuses the shared resolver
  visibility and source diagnostic helpers.
- Resolver absent value-signature metadata validation now reuses the shared
  absent-metadata entry helper.
- Resolver absent-metadata entry validation now carries diagnostic message
  formatting through an explicit absent metadata entry object, covered by
  `typechecker::tests::absent_metadata_entry_formats_message`.
- Resolver absent-metadata validation bundles now return typed absent metadata
  entries directly instead of raw present/code/label tuples, covered by
  `typechecker::tests::value_signature_absence_validation_builds_entries` and
  the related absence validation entry tests.
- Resolver absent value-signature detail validation now shares the same helper
  across module, import, local, type-like, and variant symbols.
- Resolver absent value-signature metadata validation now lets the validation
  bundle build its parameter and return metadata entries, covered by
  `typechecker::tests::value_signature_absence_validation_builds_entries`.
- Resolver module value-signature absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::value_signature_absence_validation_uses_module_resolver_codes`.
- Resolver import value-signature absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::value_signature_absence_validation_uses_import_resolver_codes`.
- Resolver local value-signature absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::value_signature_absence_validation_uses_local_resolver_codes`.
- Resolver type-like value-signature absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::value_signature_absence_validation_uses_type_like_resolver_codes`.
- Resolver variant value-signature absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::value_signature_absence_validation_uses_variant_resolver_codes`.
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
- Resolver module behavior-association absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::behavior_association_absence_validation_uses_module_resolver_codes`.
- Resolver import behavior-association absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::behavior_association_absence_validation_uses_import_resolver_codes`.
- Resolver local behavior-association absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::behavior_association_absence_validation_uses_local_resolver_codes`.
- Resolver variant behavior-association absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::behavior_association_absence_validation_uses_variant_resolver_codes`.
- Resolver behavior-symbol behavior-association absence validation now owns its
  resolver diagnostic code mapping, covered by
  `typechecker::tests::behavior_association_absence_validation_uses_behavior_resolver_codes`.
- Resolver value behavior-association absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::behavior_association_absence_validation_uses_value_resolver_codes`.
- Resolver absent behavior-declaration metadata validation now shares one
  helper across module, import, local, variant, and value symbols.
- Resolver absent behavior-declaration metadata validation now lets the
  validation bundle build its method/parent metadata entries, covered by
  `typechecker::tests::behavior_declaration_absence_validation_builds_entries`.
- Resolver module behavior-declaration absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::behavior_declaration_absence_validation_uses_module_resolver_codes`.
- Resolver import behavior-declaration absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::behavior_declaration_absence_validation_uses_import_resolver_codes`.
- Resolver local behavior-declaration absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::behavior_declaration_absence_validation_uses_local_resolver_codes`.
- Resolver variant behavior-declaration absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::behavior_declaration_absence_validation_uses_variant_resolver_codes`.
- Resolver value behavior-declaration absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::behavior_declaration_absence_validation_uses_value_resolver_codes`.
- Resolver absent mutability metadata validation now shares one helper across
  module, import, type-like, variant, and value symbols.
- Generic type substitution now covers mutable pointers, raw pointers, slices,
  arrays, and function signatures so Phase 5 specializations do not leave
  nested type parameters inside composite type shapes, covered by
  `typechecker::tests::substitute_type_covers_all_composite_type_shapes`.
- Generic function-type substitutions now round-trip through nested generic
  type arguments instead of degrading to `void`, covered by
  `typechecker::tests::substitute_type_preserves_function_type_arguments_in_nested_generics`.
- Generic method call arity diagnostics now preserve method wording through
  the shared call-signature checker, covered by
  `generic_diagnostics::generic_method_argument_arity_uses_method_diagnostic`.
- Generic enum method specialization now covers multiple concrete
  instantiations in one program for both `Option<T>` and `Result<T, E>`,
  covered by `generic_enum_multi_specialization`,
  `generic_result_enum_multi_specialization`, and the generated-C assertions in
  `generic_specializations_do_not_emit_unspecialized_c_symbols`.
- Explicit generic function and method type-argument arity failures now stop
  before specialization emits misleading follow-up inference diagnostics,
  covered by
  `generic_diagnostics::generic_function_explicit_type_arg_arity_does_not_emit_inference_followup`
  and
  `generic_diagnostics::generic_method_explicit_type_arg_arity_does_not_emit_inference_followup`.
- Invalid explicit generic function and method type-argument arity now also
  skips dependent signature checks so bare omitted type parameters do not
  cascade into argument or return mismatches, covered by
  `generic_diagnostics::generic_function_explicit_type_arg_arity_does_not_emit_argument_followup`
  and
  `generic_diagnostics::generic_method_explicit_type_arg_arity_does_not_emit_argument_followup`.
- Imported generic enum method explicit type-argument arity failures use the
  same hard diagnostic and suppress inference/argument followups through the
  module graph, covered by
  `integration::imported_generic_enum_method_explicit_type_arg_arity_is_error`.
- Imported generic function explicit type-argument arity failures use the same
  hard diagnostic and suppress inference/argument followups through the module
  graph, covered by
  `integration::imported_generic_function_explicit_type_arg_arity_is_error`.
- Malformed nested generic type annotations inside explicit function and
  method call type arguments now also skip dependent signature checks, covered
  by `generic_diagnostics::generic_function_type_arg_annotation_arity_is_error`
  and `generic_diagnostics::generic_method_type_arg_annotation_arity_is_error`.
- Generic behavior bound failures now skip dependent function and method body
  specialization diagnostics, covered by strengthened assertions in
  `generic_diagnostics::generic_function_behavior_bound_failure_is_error`,
  `generic_diagnostics::generic_method_behavior_bound_failure_is_error`, and
  `generic_diagnostics::generic_ufc_function_behavior_bound_failure_is_error`.
- Resolver value-parameter validation now owns its resolver diagnostic code
  mapping, covered by
  `typechecker::tests::value_parameter_validation_uses_resolver_codes`.
- Resolver module type-parameter absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::type_parameter_absence_validation_uses_module_resolver_codes`.
- Resolver import type-parameter absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::type_parameter_absence_validation_uses_import_resolver_codes`.
- Resolver local type-parameter absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::type_parameter_absence_validation_uses_local_resolver_codes`.
- Resolver variant type-parameter absence validation now owns its resolver
  diagnostic code mapping, covered by
  `typechecker::tests::type_parameter_absence_validation_uses_variant_resolver_codes`.
- Resolver module field absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::field_absence_validation_uses_module_resolver_codes`.
- Resolver import field absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::field_absence_validation_uses_import_resolver_codes`.
- Resolver local field absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::field_absence_validation_uses_local_resolver_codes`.
- Resolver type-like field absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::field_absence_validation_uses_type_like_resolver_codes`.
- Resolver variant field absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::field_absence_validation_uses_variant_resolver_codes`.
- Resolver behavior field absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::field_absence_validation_uses_behavior_resolver_codes`.
- Resolver value field absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::field_absence_validation_uses_value_resolver_codes`.
- Resolver module variant absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::variant_absence_validation_uses_module_resolver_codes`.
- Resolver import variant absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::variant_absence_validation_uses_import_resolver_codes`.
- Resolver local variant absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::variant_absence_validation_uses_local_resolver_codes`.
- Resolver type-like variant absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::variant_absence_validation_uses_type_like_resolver_codes`.
- Resolver behavior variant absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::variant_absence_validation_uses_behavior_resolver_codes`.
- Resolver value variant absence validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::variant_absence_validation_uses_value_resolver_codes`.
- Resolver type-like type-parameter validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::type_parameter_validation_uses_type_like_resolver_codes`.
- Resolver value type-parameter validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::type_parameter_validation_uses_value_resolver_codes`.
- Resolver value parameter-count validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::count_validation_uses_value_parameter_resolver_code`.
- Resolver field-count validation now owns its resolver diagnostic code mapping,
  covered by `typechecker::tests::count_validation_uses_field_resolver_code`.
- Resolver variant payload-count validation now owns its resolver diagnostic
  code mapping, covered by
  `typechecker::tests::count_validation_uses_variant_payload_resolver_code`.
- Resolver absent-metadata validation now routes value signature, type
  parameter, field, variant, behavior association, behavior declaration, and
  mutability entry builders through one typed replay helper, covered by
  `cargo test absence_validation`.
- Resolver type-parameter and value-parameter validation now share one
  metadata-list comparison helper for names, display metadata, and typed
  metadata, covered by
  `cargo test type_parameter_validation` and
  `cargo test value_parameter_validation`.
- Resolver behavior-method validation now uses the shared metadata-list
  comparison helper for display signatures and typed method metadata, covered
  by `cargo test behavior_method_validation`.
- Resolver field validation now uses the shared metadata-list comparison
  helper for display field metadata and typed field metadata, covered by
  `cargo test field_validation` and
  `cargo test check_program_with_symbols_validates_resolver_struct_field`.
- Resolver variant payload validation now uses a shared optional-metadata
  comparison helper for display payload metadata and typed payload metadata,
  covered by `cargo test variant_payload_validation`,
  `cargo test check_program_with_symbols_validates_resolver_enum_variant_payload`,
  and `cargo test check_program_with_symbols_validates_resolver_enum_typed_payload_metadata`.
- Resolver value return validation now uses the shared optional-metadata
  comparison helper for display return metadata and typed return metadata,
  covered by `cargo test return_validation`,
  `cargo test check_program_with_symbols_validates_resolver_function_return_type`,
  `cargo test check_program_with_symbols_validates_resolver_function_type_return_metadata`,
  and `cargo test check_program_with_symbols_validates_resolver_function_typed_signature_metadata`.
- Resolver enum variant-name validation now uses the shared metadata-list
  comparison helper for resolver-owned variant names, covered by
  `cargo test variant_name_validation` and
  `cargo test check_program_with_symbols_validates_resolver_enum_variant_names`.
- Resolver enum variant owner-name validation now uses the shared
  optional-metadata comparison helper for resolver-owned owner names, covered by
  `cargo test variant_owner_validation` and
  `cargo test check_program_with_symbols_validates_resolver_enum_variant_owner_names`.
- Resolver declaration metadata task collection now gathers callable, type,
  behavior, and behavior-impl block tasks in one declaration pass before
  replaying the existing resolver-backed collection order. Type behavior-ref
  refresh now reuses the collected type tasks, covered by
  `cargo test resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations`
  and
  `cargo test collect_declarations_with_symbols_uses_resolver_behavior_impl_target_and_name_metadata`.
- Resolver-backed behavior impl semantic validation now reuses the collected
  behavior-impl block tasks instead of rebuilding impl tasks from declarations,
  covered by
  `cargo test impl_block_declaration_tasks_collect_behavior_and_plain_impls`,
  `cargo test behavior_impl_validation_tasks_collect_impl_blocks` and
  `cargo test collect_declarations_with_symbols_reports_resolver_restored_impl_target_and_name`.
- Resolver-backed behavior requires semantic validation now reuses collected
  requires tasks from the declaration metadata pass, covered by
  `cargo test behavior_requires_validation_tasks_collect_requires_declarations`,
  `cargo test collect_declarations_with_symbols_uses_restored_requires_ref_for_inherited_impl`,
  `cargo test collect_declarations_with_symbols_uses_distinct_restored_requires_type_args`,
  and
  `cargo test collect_declarations_with_symbols_does_not_validate_stale_requires_after_target_restore`.
- Resolver-backed struct field default validation now uses dedicated semantic
  validation tasks instead of carrying unused resolver metadata while replaying
  standalone default checks,
  covered by
  `cargo test resolver_struct_field_defaults_validate_from_semantic_tasks`.
  The fallback resolver-backed struct field default validation path also uses
  the focused semantic task collector, covered by
  `cargo test resolver_backed_struct_field_defaults_use_semantic_tasks`.
- Resolver-backed generic type-reference validation now reuses type-reference
  tasks from the declaration metadata pass instead of rescanning declarations
  during semantic replay, covered by
  `cargo test resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations`,
  `cargo test collect_declarations_with_symbols_uses_resolver_function_type_metadata`,
  and
  `cargo test collect_declarations_with_symbols_does_not_validate_stale_generic_function_body_refs_when_signature_incomplete`.
  The fallback resolver-backed type-reference validation path now also routes
  through the same declaration metadata task collector, preserving top-level
  expression validation coverage with
  `cargo test resolver_declaration_metadata_tasks_collect_top_level_type_reference_tasks`.
- Resolver behavior declaration metadata now uses a named metadata task instead
  of anonymous tuple fields, covered by
  `cargo test resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations`,
  `cargo test collect_declarations_with_symbols_uses_resolver_behavior_method_metadata`,
  and
  `cargo test collect_declarations_with_symbols_clears_stale_behavior_methods_after_name_restore`.
- Behavior impl conformance now carries effective impl methods as named
  declaration/name records instead of positional tuples, covered by
  `cargo test effective_behavior_impl_methods_carry_named_declaration_and_method_name`,
  `cargo test impl_effective_method_name_prefers_resolver_then_ast_then_collected_signature`,
  and
  `cargo test collect_declarations_with_symbols_uses_resolver_behavior_impl_method_signature_target_and_name_metadata`.
- Behavior impl method naming, resolver-owned key restoration, and
  behavior-specialized method-key helpers now live in
  `src/typechecker/behavior_impl_methods.rs`, reducing
  `src/typechecker/behavior_impl_support.rs` to 385 lines while preserving
  the focused impl-method helper tests above.
- Generic template dependency save/restore entries now use named `name` and
  `previous` fields instead of raw `(name, previous)` tuples in the
  monomorphization dependency snapshots, covered by
  `cargo test template_dependency_entries_use_named_fields`,
  `cargo test check_module_graph_entry_specializes_imported_generic_functions`,
  and
  `cargo test check_module_graph_entry_specializes_public_generic_methods_for_imported_types`.
- Callable declaration collection now uses named function and method tasks
  before replaying AST or resolver-backed collection, covered by
  `cargo test callable_declaration_tasks_collect_functions_and_methods`,
  `cargo test collect_declarations_with_symbols_uses_resolver_function_type_metadata`,
  and
  `cargo test collect_declarations_with_symbols_uses_resolver_method_signature_for_type_refs`.
- AST type declaration collection now uses named struct and enum tasks before
  replaying generic-bound validation and type registration, covered by
  `cargo test ast_type_declaration_tasks_collect_structs_and_enums`,
  `cargo test generic_struct_constructor_without_type_args_is_error`,
  `cargo test nongeneric_struct_constructor_type_args_are_error`,
  `cargo test nongeneric_enum_constructor_type_args_are_error`,
  `cargo test nongeneric_struct_annotation_type_args_are_error`,
  `cargo test nongeneric_enum_annotation_type_args_are_error`,
  `cargo test behavior_impl_nongeneric_behavior_type_args_are_error`,
  `cargo test behavior_requires_nongeneric_behavior_type_args_are_error`,
  `cargo test behavior_extends_nongeneric_parent_type_args_are_error`,
  `cargo test generic_bound_nongeneric_behavior_type_args_are_error`,
  `cargo test generic_enum_type_arg_arity_is_error`,
  `cargo test generic_enum_constructor_without_type_args_is_error`, and
  `cargo test check_program_with_symbols_uses_resolver_type_metadata_for_type_refs`.
- Behavior declaration collection now uses named behavior tasks before
  replaying AST signature registration or resolver-backed stubs, covered by
  `cargo test behavior_declaration_tasks_collect_behavior_signatures`,
  `cargo test behavior_declaration_collection`, and
  `cargo test resolver_backed_behavior_collection_defers_generic_metadata_to_resolver`.
- AST import declaration collection now uses named import tasks before seeding
  imported names into the typechecker import table, covered by
  `cargo test ast_import_declaration_tasks_collect_import_bindings`,
  `cargo test collect_import_info`, and
  `cargo test check_program_with_symbols_uses_resolver_import_bindings`.
- AST struct field-default validation now uses named struct tasks before
  replaying nongeneric default expression checks, covered by
  `cargo test ast_struct_field_default_validation_tasks_collect_structs`,
  `cargo test check_program_rejects_struct_field_default_type_mismatch`,
  and
  `cargo test check_program_rejects_unknown_type_references_in_struct_field_defaults`.
- AST generic type-reference validation now uses named declaration tasks before
  replaying declaration-specific type and expression reference checks, covered
  by `cargo test ast_type_reference_validation_tasks_collect_declarations`,
  `cargo test check_program_rejects_unknown_type_references`, and
  `cargo test check_program_rejects_unknown_type_references_in_struct_field_defaults`.
- Self-type context validation now uses named declaration tasks before
  replaying declaration-specific `Self` allowance checks, covered by
  `cargo test self_type_context_validation_tasks_collect_declarations`,
  `cargo test check_program_rejects_self_type_outside_method_or_behavior`,
  and `cargo test behavior_declaration_collection`.
- Resolver validation replay now separates declaration replay collection from
  final behavior-association task construction, covered by
  `cargo test resolver_validation_replay_declaration_tasks_collect_sources_and_edges`,
  `cargo test resolver_validation_replay_tasks_collect_symbols_and_behavior_associations_together`,
  and
  `cargo test resolver_behavior_association_list_tasks_collect_type_and_parent_edges_together`.
- Resolver-backed type-reference validation now uses a narrow type-reference
  task collector instead of collecting the full resolver metadata task bundle
  when only type-reference replay is needed, covered by
  `cargo test resolver_type_reference_validation_tasks_collect_only_type_reference_work`
  and
  `cargo test collect_declarations_with_symbols_uses_resolver_type_metadata_for_type_refs`.
- Resolver-backed struct field-default validation now uses the dedicated
  semantic task collector instead of collecting the full resolver metadata
  task bundle when only default replay is needed, covered by
  `cargo test resolver_declaration_semantic_tasks_collect_only_semantic_work`,
  `cargo test resolver_backed_struct_field_defaults_use_semantic_tasks`, and
  `cargo test resolver_struct_field_defaults_validate_from_semantic_tasks`.
- Resolver-backed behavior declaration metadata now uses a narrow behavior
  metadata task collector shared by the full resolver metadata collector,
  covered by
  `cargo test resolver_behavior_declaration_metadata_tasks_collect_only_behavior_work`
  and
  `cargo test resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations`.
- Resolver-backed callable declaration metadata now uses a narrow callable
  metadata task collector shared by the full resolver metadata collector,
  covered by
  `cargo test resolver_callable_declaration_metadata_tasks_collect_callable_work`
  and
  `cargo test resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations`.
- Resolver-backed behavior impl block metadata now uses a narrow impl-block
  task collector shared by the full resolver metadata collector, covered by
  `cargo test resolver_behavior_impl_block_declaration_tasks_collect_only_behavior_impls`
  and
  `cargo test resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations`.
- Resolver-backed behavior requires validation now shares one requires-task
  push helper between the dedicated requires collector and the full resolver
  metadata collector, covered by
  `cargo test behavior_requires_validation_tasks_collect_requires_declarations`
  and
  `cargo test resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations`.
- Behavior impl validation now uses a shared impl-task push helper before
  replaying conformance checks, covered by
  `cargo test behavior_impl_validation_tasks_collect_impl_blocks`,
  `cargo test resolver_behavior_impl_block_declaration_tasks_collect_only_behavior_impls`,
  and
  `cargo test resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations`.
- Behavior extends validation now uses a shared extends-task push helper before
  replaying parent-edge checks, covered by
  `cargo test behavior_extends_validation_tasks_collect_parent_refs` and
  `cargo test behavior_extends_duplicate_parent_is_error`.
- AST struct field-default validation now uses a shared default-task push
  helper before replaying default expression checks, covered by
  `cargo test ast_struct_field_default_validation_tasks_collect_structs` and
  `cargo test check_program_rejects_struct_field_default_type_mismatch`.
- AST import declaration collection now uses a shared import-task push helper
  before seeding imported names, covered by
  `cargo test ast_import_declaration_tasks_collect_import_bindings` and
  `cargo test collect_import_info`.
- Impl-block declaration collection now uses a shared impl-block task push
  helper before replaying type and behavior impl setup, covered by
  `cargo test impl_block_declaration_tasks_collect_behavior_and_plain_impls`
  and `cargo test test_type_impl_methods`.
- Callable declaration collection now uses a shared callable-task push helper
  before replaying function and method signature setup, covered by
  `cargo test callable_declaration_tasks_collect_functions_and_methods` and
  `cargo test generic_function_collection`.
- AST type declaration collection now uses a shared type-task push helper
  before replaying generic-bound validation and type registration, covered by
  `cargo test ast_type_declaration_tasks_collect_structs_and_enums`,
  `cargo test generic_struct_constructor_without_type_args_is_error`,
  `cargo test nongeneric_struct_constructor_type_args_are_error`,
  `cargo test nongeneric_enum_constructor_type_args_are_error`,
  `cargo test nongeneric_struct_annotation_type_args_are_error`,
  `cargo test nongeneric_enum_annotation_type_args_are_error`,
  `cargo test behavior_impl_nongeneric_behavior_type_args_are_error`,
  `cargo test behavior_requires_nongeneric_behavior_type_args_are_error`,
  `cargo test behavior_extends_nongeneric_parent_type_args_are_error`,
  `cargo test generic_bound_nongeneric_behavior_type_args_are_error`,
  `cargo test generic_enum_type_arg_arity_is_error`, and
  `cargo test generic_enum_constructor_without_type_args_is_error`.
- Behavior declaration collection now uses a shared behavior-task push helper
  before replaying signature setup and behavior generic-bound validation,
  covered by `cargo test behavior_declaration_tasks_collect_behavior_signatures`
  and `cargo test behavior_generic_bound_accepts_later_behavior_declaration`.
- Self type context validation now uses a shared context-task push helper
  before replaying declaration and expression `Self` checks, covered by
  `cargo test self_type_context_validation_tasks_collect_declarations` and
  `cargo test check_program_rejects_self_type_outside_method_or_behavior`.
- AST type reference validation now uses a shared reference-task push helper
  before replaying generic type-reference diagnostics, covered by
  `cargo test ast_type_reference_validation_tasks_collect_declarations` and
  `cargo test check_program_rejects_unknown_type_references`.
- Resolver validation replay now uses shared behavior-association and parent
  list push helpers before checking resolver metadata lists, covered by
  `cargo test resolver_validation_replay_tasks_collect_symbols_and_behavior_associations_together`
  and
  `cargo test resolver_behavior_association_list_tasks_collect_type_and_parent_edges_together`.
- Resolver validation replay now uses a shared callable-symbol helper for
  top-level functions, methods, and impl methods, covered by
  `cargo test resolver_expected_symbol_sets_collect_declarations_and_locals_together`
  and
  `cargo test expected_resolver_impl_method_symbols_collect_value_symbols_and_locals`.
- Resolver validation replay now uses a shared association-source helper for
  type and behavior declaration symbols, covered by
  `cargo test resolver_validation_replay_declaration_tasks_collect_sources_and_edges`
  and
  `cargo test resolver_expected_symbol_sets_collect_declarations_and_locals_together`.
- Resolver validation replay now uses a shared import-symbol helper for
  expected module and import entries, covered by
  `cargo test resolver_expected_symbol_sets_collect_declarations_and_locals_together`
  and `cargo test ast_import_declaration_tasks_collect_import_bindings`.
- Resolver validation replay now uses shared behavior-edge helpers for impl,
  requires, and parent edges, covered by
  `cargo test expected_behavior_associations_build_impl_and_required_edges_together`,
  `cargo test resolver_validation_replay_declaration_tasks_collect_sources_and_edges`,
  and
  `cargo test resolver_behavior_association_list_tasks_collect_type_and_parent_edges_together`.
- Resolver validation replay now uses a shared variant-symbol helper for
  expected enum variant entries, covered by
  `cargo test check_program_with_symbols_requires_resolver_enum_variants` and
  `cargo test resolver_validation_replay_declaration_tasks_collect_sources_and_edges`.
- Resolver validation replay now uses a shared scoped-expression helper for
  struct field defaults and top-level expressions, covered by
  `cargo test check_program_with_symbols_requires_resolver_top_level_expr_locals`,
  `cargo test check_program_with_symbols_requires_resolver_struct_field_default_locals`,
  and `cargo test expected_resolver_scoped_expr_locals_collects_block_bindings`.
- Resolver-backed callable replay now shares one helper for callable metadata
  and callable type-reference validation tasks, covered by
  `cargo test resolver_callable_replay_task_helper_pushes_metadata_and_type_refs_together`,
  `cargo test resolver_callable_declaration_metadata_tasks_collect_callable_work`,
  and
  `cargo test resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations`.
- Resolver-backed type declaration replay now shares one helper for struct/enum
  metadata and type-reference validation tasks, covered by
  `cargo test resolver_type_replay_task_helper_pushes_metadata_and_type_refs_together`
  and `cargo test resolver_type_declaration_metadata_tasks_collect_only_type_work`.
- Resolver-backed behavior declaration replay now shares one helper for
  behavior metadata and type-reference validation tasks, covered by
  `cargo test resolver_behavior_replay_task_helper_pushes_metadata_and_type_refs_together`
  and
  `cargo test resolver_behavior_declaration_metadata_tasks_collect_only_behavior_work`.
- Resolver-backed behavior impl-block replay now shares one helper for
  behavior-impl metadata and impl-block type-reference validation tasks, covered
  by
  `cargo test resolver_behavior_impl_replay_task_helper_pushes_metadata_and_type_refs_together`
  and
  `cargo test resolver_behavior_impl_block_declaration_tasks_collect_only_behavior_impls`.
- Behavior requires replay now uses one named helper for both standalone
  semantic validation task collection and resolver-backed declaration metadata
  collection, covered by
  `cargo test behavior_requires_replay_task_helper_pushes_requires_validation`,
  `cargo test behavior_requires_validation_tasks_collect_requires_declarations`,
  and
  `cargo test resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations`.
- Behavior extends replay now uses one named helper for parent-edge semantic
  validation task collection, covered by
  `cargo test behavior_extends_replay_task_helper_pushes_parent_validation`
  and `cargo test behavior_extends_validation_tasks_collect_parent_refs`.
- Normal `zen build build.zen` now routes through the constrained
  deterministic build graph pipeline and compiles executable graph targets,
  covered by
  `cargo test --test integration build_command_routes_build_zen_through_deterministic_graph`.
  Multi-target execution is covered by
  `cargo test --test integration build_command_build_zen_compiles_multiple_executable_targets`.
  Test-only graphs are rejected before execution starts through
  `cargo test --test integration build_command_build_zen_rejects_graph_without_executable_targets`.
  The same entrypoint rejects undeclared host effects before multi-target
  execution,
  covered by
  `cargo test --test integration build_command_multi_target_build_zen_rejects_undeclared_host_effects`.
- Normal `zen check build.zen` validates the constrained deterministic graph
  without compiling targets, covered by
  `cargo test --test integration check_command_validates_build_zen_graph`, and
  rejects missing executable, test, and library sources through
  `cargo test --test integration check_command_build_zen_rejects_missing_executable_source`,
  `cargo test --test integration check_command_build_zen_rejects_missing_test_source`,
  and
  `cargo test --test integration check_command_build_zen_rejects_missing_library_source`.
  It rejects undeclared host effects before source validation through
  `cargo test --test integration check_command_build_zen_rejects_undeclared_host_effects_before_source_validation`.
  The single-target host-effect rejection remains covered by
  `cargo test --test integration check_command_build_zen_rejects_undeclared_host_effects`.
- Normal `zen test build.zen` compiles and runs test graph targets, covered by
  `cargo test --test integration test_command_build_zen_runs_test_targets`,
  compiles and runs multiple test graph targets through
  `cargo test --test integration test_command_build_zen_runs_multiple_test_targets`,
  runs test target dependencies before dependents through
  `cargo test --test integration test_command_build_zen_runs_test_dependencies_first`,
  rejects executable-only graphs before execution starts through
  `cargo test --test integration test_command_build_zen_rejects_graph_without_test_targets`,
  and rejects undeclared host effects before test execution through
  `cargo test --test integration test_command_build_zen_rejects_undeclared_host_effects`
  and
  `cargo test --test integration test_command_multi_target_build_zen_rejects_undeclared_host_effects`.
- Normal `zen emit build.zen` emits generated C for the single executable graph
  target without compiling a binary, covered by
  `cargo test --test integration emit_command_build_zen_outputs_target_c_source`,
  rejects ambiguous zero-target and multi-executable C emission through
  `cargo test --test integration emit_command_build_zen_rejects_graph_without_executable_targets`
  and
  `cargo test --test integration emit_command_build_zen_rejects_multiple_executable_targets`,
  and rejects undeclared host effects through
  `cargo test --test integration emit_command_build_zen_rejects_undeclared_host_effects`.
- Direct `zen build.zen` aliases the same constrained deterministic graph build
  path as `zen build build.zen`, covered by
  `cargo test --test integration direct_file_command_build_zen_routes_through_deterministic_graph`,
  compiles multiple executable targets through
  `cargo test --test integration direct_file_command_build_zen_compiles_multiple_executable_targets`,
  compiles executable dependencies before dependents through
  `cargo test --test integration direct_file_command_build_zen_compiles_executable_dependencies_first`,
  accepts validated library source dependencies and rejects gated test
  dependencies through
  `cargo test --test integration direct_file_command_build_zen_accepts_library_dependencies`
  and
  `cargo test --test integration direct_file_command_build_zen_rejects_gated_test_dependencies`,
  rejects test-only graphs before execution starts through
  `cargo test --test integration direct_file_command_build_zen_rejects_graph_without_executable_targets`,
  and rejects undeclared host effects for single-target and multi-target graphs
  through
  `cargo test --test integration direct_file_command_multi_target_build_zen_rejects_undeclared_host_effects`
  and
  `cargo test --test integration direct_file_command_build_zen_rejects_undeclared_host_effects`.
- Build script lowering collects multiple executable targets deterministically,
  covered by
  `cargo test --test build_graph build_program_lowering_collects_multiple_executable_targets`.
- Build script lowering includes checked-in project test targets and standalone
  `Test { root: ... }` targets, covered by
  `cargo test --test build_graph parsed_project_build_zen_lowers_to_executable_and_test_graph`
  and `cargo test --test build_graph build_program_lowering_collects_test_target`.
- Build script lowering includes graph-only library targets, covered by
  `cargo test --test build_graph build_program_lowering_collects_library_target`.
- Build script lowering includes target dependency and feature metadata arrays,
  covered by
  `cargo test --test build_graph build_program_lowering_collects_target_dependencies_and_features`.
- Build script lowering recognizes declared deterministic file-read effects
  without allowing undeclared file reads, covered by
  `cargo test --test build_graph file_reads`.
- Build script lowering keeps accepted `build.zen` DSL spellings owned by
  focused enums for target kinds, target fields, and builder identifiers,
  covered by `cargo test build_target_dsl --lib` and
  `cargo test build_target_field_owns_source_spelling --lib`.
  Those spelling guards now live in `src/build_graph/lowering_tests.rs`,
  keeping `src/build_graph/lowering.rs` below the cleanup threshold.
- Build-command integration coverage now keeps ordinary build diagnostics,
  `build.zen` graph validation, and `build.zen` host-effect ordering in
  focused modules, with the split guarded by the same full integration suite
  and focused filters for the moved validation and host-effect cases.
- `zen test build.zen` integration coverage now keeps execution/order tests,
  host-effect rejection, and graph validation failures in focused modules,
  with focused filters for moved execution, host-effect, and validation cases
  plus the full integration suite preserving behavior.
- Build graph validation rejects unresolved target dependencies, covered by
  `cargo test --test build_graph build_graph_rejects_unknown_target_dependencies`
  and
  `cargo test --test build_graph build_program_lowering_rejects_unknown_target_dependencies`.
- Build graph lowering rejects unsupported package/link targets with targeted
  diagnostic instead of silently treating them as absent, covered by
  `cargo test --test build_graph build_program_lowering_rejects_unsupported_package_targets`
  and
  `cargo test --test build_graph build_program_lowering_rejects_unsupported_link_targets`.
- Build graph lowering accepts declared deterministic env-read effects before
  graph promotion, covered by
  `cargo test --test build_graph build_program_lowering_accepts_declared_env_reads`.
- Build target kind spellings are owned by enums instead of duplicated in
  semantic and CLI logic. `build_target_dsl_kind_owns_source_spelling` covers
  accepted build target DSL names and the supported-target diagnostic list, and
  `build_target_kind_owns_diagnostic_spelling` covers runtime target-kind
  diagnostic names used by gated dependency errors.
- Build graph validation rejects self-dependencies, covered by
  `cargo test --test build_graph build_graph_rejects_self_target_dependencies`
  and
  `cargo test --test build_graph build_program_lowering_rejects_self_target_dependencies`.
- `emit-json build-graph` keeps host-effect rejection ahead of test target graph
  emission, covered by
  `cargo test --test integration emit_json_build_graph_rejects_undeclared_host_effects_before_test_target_lowering`.
- `emit-json build-graph` emits graph-only library targets and keeps host-effect
  rejection ahead of library target graph emission, covered by
  `cargo test --test integration emit_json_build_graph_outputs_library_target`
  and `cargo test --test integration emit_json_build_graph_rejects_undeclared_host_effects_before_library_target_lowering`.
- `emit-json build-graph` emits target dependency and feature metadata arrays
  and keeps host-effect rejection ahead of target metadata graph emission,
  covered by
  `cargo test --test integration emit_json_build_graph_outputs_target_dependencies_and_features`
  and `cargo test --test integration emit_json_build_graph_rejects_undeclared_host_effects_before_target_metadata_lowering`.
- `emit-json build-graph` emits declared deterministic file-read effects and
  rejects undeclared file reads through the advertised graph command, covered
  by `cargo test --test integration file_read_effects`.
- `emit-json build-graph` emits declared deterministic env-read effects
  through the advertised graph command, covered by
  `cargo test --test integration emit_json_build_graph_outputs_declared_env_read_effects`.
- `emit-json build-graph` rejects unresolved target dependencies through the
  advertised graph-emission path, covered by
  `cargo test --test integration emit_json_build_graph_rejects_unknown_target_dependencies`.
- `emit-json build-graph` rejects unsupported package/link targets through the
  advertised graph-emission path, covered by
  `cargo test --test integration emit_json_build_graph_rejects_unsupported_package_targets`
  and
  `cargo test --test integration emit_json_build_graph_rejects_unsupported_link_targets`.
  Normal build, direct build.zen execution, check, test, emit, and legacy
  build-graph entrypoints now have the same unsupported package/link gate
  coverage before execution or emission through
  `build_command_build_zen_rejects_unsupported_package_targets`,
  `build_command_build_zen_rejects_unsupported_link_targets`,
  `direct_file_command_build_zen_rejects_unsupported_package_targets`,
  `direct_file_command_build_zen_rejects_unsupported_link_targets`,
  `check_command_build_zen_rejects_unsupported_package_targets`,
  `check_command_build_zen_rejects_unsupported_link_targets`,
  `test_command_build_zen_rejects_unsupported_package_targets`,
  `test_command_build_zen_rejects_unsupported_link_targets`,
  `emit_command_build_zen_rejects_unsupported_package_targets`,
  `emit_command_build_zen_rejects_unsupported_link_targets`,
  `build_graph_command_rejects_unsupported_package_targets`, and
  `build_graph_command_rejects_unsupported_link_targets`.
- `emit-json build-graph` rejects self-dependencies through the advertised
  graph-emission path, covered by
  `cargo test --test integration emit_json_build_graph_rejects_self_target_dependencies`.
- Build graph validation rejects dependency cycles, covered by
  `cargo test --test build_graph build_graph_rejects_cyclic_target_dependencies`
  and
  `cargo test --test build_graph build_program_lowering_rejects_cyclic_target_dependencies`.
- `emit-json build-graph` rejects dependency cycles through the advertised
  graph-emission path, covered by
  `cargo test --test integration emit_json_build_graph_rejects_cyclic_target_dependencies`.
- Legacy `emit-json ast|symbols|typed|diagnostics build.zen` modes remain
  rejected with a targeted graph diagnostic, covered by
  `cargo test --test integration legacy_emit_json_modes_reject_build_zen_with_graph_diagnostic`.
- `build-graph <build.zen>` consumes the deterministic graph for executable
  targets and rejects undeclared host effects before execution starts, covered
  by
  `cargo test --test integration build_graph_command_compiles_single_executable_target`
  and
  `cargo test --test integration build_graph_command_compiles_multiple_executable_targets`.
  Dependency-ordered executable target execution is covered by
  `cargo test --test integration build_graph_command_compiles_executable_dependencies_first`.
  Single-target and multi-target host-effect rejection are covered by
  `cargo test --test integration build_graph_command_rejects_undeclared_host_effects`
  and
  `cargo test --test integration build_graph_command_multi_target_rejects_undeclared_host_effects`.
  Declared deterministic env reads with fallbacks are accepted through
  `cargo test --test integration build_graph_command_accepts_declared_env_read_with_fallback`.
  Declared deterministic file-read effects are accepted and undeclared file
  reads reject before execution, covered by
  `cargo test --test integration build_graph_command_accepts_declared_file_read_effects`
  and
  `cargo test --test integration build_graph_command_rejects_undeclared_file_read_effects_before_execution`.
  Graph-only library export source validation and typechecking before
  execution are covered by
  `cargo test --test integration build_graph_command_rejects_missing_graph_only_library_source`,
  `cargo test --test integration build_graph_command_accepts_valid_graph_only_library_sources`,
  `cargo test --test integration build_graph_command_rejects_graph_only_library_type_errors`,
  and
  `cargo test --test integration build_graph_command_rejects_undeclared_host_effects_before_library_typechecking`.
  Test-only graphs are rejected before execution starts, covered by
  `cargo test --test integration build_graph_command_rejects_graph_without_executable_targets`.
- Executable build graph targets now execute dependencies before dependents,
  covered by
  `cargo test --test build_graph build_graph_orders_targets_before_dependents`
  and
  `cargo test --test integration build_command_build_zen_compiles_executable_dependencies_first`.
- Normal build graph execution accepts declared deterministic file-read effects
  and rejects undeclared file reads before target execution, covered by
  `cargo test --test integration build_command_build_zen_accepts_declared_file_read_effects`
  and
  `cargo test --test integration build_command_build_zen_rejects_undeclared_file_read_effects_before_execution`.
  Declared deterministic env reads with fallbacks are accepted on the same
  execution path through
  `cargo test --test integration build_command_build_zen_accepts_declared_env_read_with_fallback`.
- Build/test/legacy graph execution accepts dependencies on validated library
  source targets,
  covered by
  `cargo test --test integration build_command_build_zen_accepts_library_dependencies`,
  `cargo test --test integration build_graph_command_accepts_library_dependencies`,
  and
  `cargo test --test integration test_command_build_zen_accepts_library_dependencies`.
  Cross-mode execution gating is also covered by
  `cargo test --test integration build_command_build_zen_rejects_gated_test_dependencies`,
  `cargo test --test integration build_graph_command_rejects_gated_test_dependencies`,
  and
  `cargo test --test integration test_command_build_zen_rejects_gated_executable_dependencies`.
- Normal test graph execution accepts declared deterministic file-read effects
  and rejects undeclared file reads before test execution, covered by
  `cargo test --test integration test_command_build_zen_accepts_declared_file_read_effects`
  and
  `cargo test --test integration test_command_build_zen_rejects_undeclared_file_read_effects_before_execution`.
  Declared deterministic env reads with fallbacks are accepted on the same
  test execution path through
  `cargo test --test integration test_command_build_zen_accepts_declared_env_read_with_fallback`.
- Build/test/emit graph execution validates non-executed graph-only library
  exports before compiling or running selected targets, covered by
  `cargo test --test integration build_command_build_zen_rejects_missing_graph_only_library_source`,
  `cargo test --test integration direct_file_command_build_zen_rejects_missing_graph_only_library_source`,
  `cargo test --test integration build_graph_command_rejects_missing_graph_only_library_source`,
  `cargo test --test integration test_command_build_zen_rejects_missing_graph_only_library_source`,
  and
  `cargo test --test integration emit_command_build_zen_rejects_missing_graph_only_library_source`.
  Graph-only library exports are also typechecked before build/test/emit
  execution starts, covered by
  `cargo test --test integration build_command_build_zen_accepts_valid_graph_only_library_sources`,
  `cargo test --test integration build_command_build_zen_rejects_graph_only_library_type_errors`,
  `cargo test --test integration build_command_build_zen_rejects_undeclared_host_effects_before_library_typechecking`,
  `cargo test --test integration test_command_build_zen_accepts_valid_graph_only_library_sources`,
  `cargo test --test integration test_command_build_zen_rejects_graph_only_library_type_errors`,
  `cargo test --test integration test_command_build_zen_rejects_undeclared_host_effects_before_library_typechecking`,
  `cargo test --test integration emit_command_build_zen_accepts_valid_graph_only_library_sources`,
  and
  `cargo test --test integration emit_command_build_zen_rejects_graph_only_library_type_errors`.
- `zen check build.zen` typechecks graph target sources after deterministic
  graph/source validation and before reporting the graph summary, covered by
  `cargo test --test integration check_command_build_zen_typechecks_target_sources`.
  Library-only graphs are valid on this non-executing validation path, covered
  by
  `cargo test --test integration check_command_build_zen_accepts_library_only_graph_validation`.
  Undeclared host effects still reject before target source typechecking,
  covered by
  `cargo test --test integration check_command_build_zen_rejects_undeclared_host_effects_before_target_typechecking`.
- `zen check build.zen` accepts declared deterministic file-read effects and
  rejects undeclared file reads before source validation, covered by
  `cargo test --test integration check_command_build_zen_accepts_declared_file_read_effects`
  and
  `cargo test --test integration check_command_build_zen_rejects_undeclared_file_read_effects_before_source_validation`.
  Declared deterministic env reads with fallbacks are accepted on the same
  validation path through
  `cargo test --test integration check_command_build_zen_accepts_declared_env_read_with_fallback`.
- Single-target `zen emit build.zen` rejects multi-executable ambiguity before
  per-executable source validation, covered by
  `cargo test --test integration emit_command_build_zen_reports_multi_target_ambiguity_before_missing_executable_source`.
  It also rejects multi-executable ambiguity before graph-only library
  typechecking, covered by
  `cargo test --test integration emit_command_build_zen_reports_multi_target_ambiguity_before_graph_only_library_typechecking`.
- Single-target `zen emit build.zen` accepts declared deterministic file-read
  effects and rejects undeclared file reads before C emission, covered by
  `cargo test --test integration emit_command_build_zen_accepts_declared_file_read_effects`
  and
  `cargo test --test integration emit_command_build_zen_rejects_undeclared_file_read_effects`.
  Declared deterministic env reads with fallbacks are accepted on the same emit
  path through
  `cargo test --test integration emit_command_build_zen_accepts_declared_env_read_with_fallback`.
- Single-target `zen emit build.zen` accepts selected executable dependencies
  on validated library source targets and rejects gated test dependencies
  before C emission, covered by
  `cargo test --test integration emit_command_build_zen_accepts_library_dependencies`
  and
  `cargo test --test integration emit_command_build_zen_rejects_gated_test_dependencies`.
- Single-target `zen emit build.zen` validates and typechecks graph-only
  library exports before emission, covered by
  `cargo test --test integration emit_command_build_zen_accepts_valid_graph_only_library_sources`,
  `cargo test --test integration emit_command_build_zen_rejects_graph_only_library_type_errors`,
  and
  `cargo test --test integration emit_command_build_zen_rejects_undeclared_host_effects_before_library_typechecking`.
- Library-only build graphs remain non-executable across the current execution
  entrypoints, covered by
  `cargo test --test integration build_command_build_zen_rejects_library_only_graph_execution`,
  `cargo test --test integration direct_file_command_build_zen_rejects_library_only_graph_execution`,
  `cargo test --test integration build_graph_command_rejects_library_only_graph_execution`,
  `cargo test --test integration emit_command_build_zen_rejects_library_only_graph_execution`,
  and
  `cargo test --test integration test_command_build_zen_rejects_library_only_graph_execution`.
- Direct `zen build.zen` accepts declared deterministic file-read effects and
  rejects undeclared file reads before execution, covered by
  `cargo test --test integration direct_file_command_build_zen_accepts_declared_file_read_effects`
  and
  `cargo test --test integration direct_file_command_build_zen_rejects_undeclared_file_read_effects_before_execution`.
  Declared deterministic env reads with fallbacks are accepted on the same
  direct execution path through
  `cargo test --test integration direct_file_command_build_zen_accepts_declared_env_read_with_fallback`.
- Direct `zen build.zen` validates and typechecks graph-only library exports
  before execution, covered by
  `cargo test --test integration direct_file_command_build_zen_accepts_valid_graph_only_library_sources`,
  `cargo test --test integration direct_file_command_build_zen_rejects_graph_only_library_type_errors`,
  and
  `cargo test --test integration direct_file_command_build_zen_rejects_undeclared_host_effects_before_library_typechecking`.
- Dependency-ordered build execution still stops before execution when graph
  lowering detects undeclared host effects, covered by
  `cargo test --test integration build_command_build_zen_rejects_undeclared_host_effects_before_dependency_execution`.
- `src/typechecker/mod.rs` no longer owns resolver metadata restoration
  mechanics directly. Struct field, enum variant, behavior method, callable
  parameter, and optional-return restoration helpers live in
  `src/typechecker/resolver_metadata_collection.rs`, reducing the root module
  to typechecker state/orchestration while retaining focused resolver metadata
  restoration coverage such as
  `cargo test --lib resolver_behavior_methods_from_metadata_preserves_defaults_by_resolver_order`.
- `src/cli.rs` no longer owns diagnostic rendering directly. Compile-error and
  diagnostic printing helpers live in `src/cli/diagnostics.rs`, reducing the
  root CLI module below the 500-line cleanup threshold while preserving
  command diagnostic coverage through
  `cargo test --test integration cli_build::diagnostics`.
- Resolver behavior-ref metadata validation now lives in
  `src/typechecker/resolver_validation/metadata_behavior_refs.rs`, reducing
  `metadata_types.rs` to type, field, variant, and behavior-method metadata
  checks while preserving parent/impl/requires edge coverage through
  `cargo test --lib resolver_validation::behavior_refs`.
- Expression method-call checking now lives in
  `src/typechecker/expressions/method_call_support.rs`, reducing
  `call_support.rs` to direct function-call handling while preserving generic
  method and UFC coverage through `cargo test --lib generic_method` and the
  `single_file_fixtures::test_generic_method*` integration filters.
- Expression function checking now lives in
  `src/typechecker/expressions/function_checking.rs`, reducing
  `expressions.rs` below the 500-line cleanup threshold while preserving
  return/fallthrough and defer behavior through
  `cargo test --lib return_type_mismatch_error`,
  `cargo test --lib non_void_function_without_return_is_error`, and
  `cargo test --test integration runtime_fixtures::test_defer_early_return`.
- `src/resolver/symbol_table_test_support.rs` now shares one test-only symbol
  lookup helper across metadata setters, reducing the file from 514 to 405
  lines while preserving resolver-backed metadata coverage through
  `cargo test --lib resolver_ -- --nocapture` and
  `cargo test --test resolver_phase2`.
- Resolver type/behavior metadata tests now keep behavior-method metadata cases
  in `src/typechecker/tests/resolver_type_behavior_metadata/behavior_methods.rs`,
  reducing `src/typechecker/tests/resolver_type_behavior_metadata.rs` from 510
  to 332 lines while preserving coverage through
  `cargo test --lib resolver_type_behavior_metadata`.
- Import-visibility integration tests now keep transitive dependency and
  private imported-method cases in focused modules, reducing
  `tests/integration/import_visibility.rs` from 500 to 161 lines while
  preserving coverage through
  `cargo test --test integration import_visibility`.
- Legacy build-graph integration tests now keep graph validation cases in
  `tests/integration/cli_build/legacy_graph_command_validation.rs`, reducing
  `tests/integration/cli_build/legacy_graph_command.rs` from 492 to 255 lines
  while preserving coverage through
  `cargo test --test integration build_graph_command`.
- Direct build.zen integration tests now keep graph validation cases in
  `tests/integration/cli_build/direct_build_graph_validation.rs`, reducing
  `tests/integration/cli_build/direct_build_graph_execution.rs` from 417 to
  198 lines while preserving coverage through
  `cargo test --test integration direct_file_command_build_zen`.
- Emit build.zen integration tests now keep graph validation cases in
  `tests/integration/cli_build/emit_direct_validation.rs`, reducing
  `tests/integration/cli_build/emit_direct.rs` from 377 to 47 lines while
  preserving coverage through
  `cargo test --test integration emit_command_build_zen`.
- Resolver import metadata tests now keep module-symbol metadata cases in
  `src/typechecker/tests/resolver_import_metadata/module_metadata.rs`,
  reducing `src/typechecker/tests/resolver_import_metadata.rs` from 506 to 280
  lines while preserving coverage through
  `cargo test --lib resolver_import_metadata`.
- Module-system tests now keep module-graph loading cases in
  `src/module_system/tests/graph_loading.rs`, reducing
  `src/module_system/tests.rs` from 503 to 333 lines while preserving coverage
  through `cargo test --lib module_system::tests`.
- Resolver behavior-association list replay now selects from the resolver
  declaration task bundle internally, covered by
  `cargo test --lib resolver_behavior_association_list_tasks_select_from_declaration_bundle`.
- AST declaration collection now replays the full collection task bundle through
  one helper, covered by
  `cargo test --lib ast_declaration_collection_bundle_replays_collection_passes`.
- AST declaration semantic validation now replays its full validation task
  bundle through one helper, covered by
  `cargo test --lib ast_declaration_semantic_bundle_replays_validation_passes`.
- Resolver-backed declaration semantic validation now replays behavior
  associations, resolver-owned type-reference checks, and resolver struct-field
  defaults through one task-bundle helper, covered by
  `cargo test --lib resolver_declaration_semantic_bundle_replays_validation_passes`.
- Resolver-backed standalone semantic validation now builds a dedicated
  semantic task bundle without carrying unused callable or behavior metadata,
  covered by
  `cargo test --lib resolver_declaration_semantic_tasks_collect_only_semantic_work`
  and
  `cargo test --lib resolver_declaration_semantic_bundle_replays_dedicated_semantic_tasks`.
- Resolver-backed declaration collection now replays resolver metadata
  restoration, behavior-impl metadata restoration, semantic validation,
  behavior-ref cleanup, and final type behavior refresh through one
  task-bundle helper, covered by
  `cargo test --lib resolver_declaration_collection_bundle_replays_metadata_semantics_and_refresh`.
- Resolver-backed declaration collection now builds AST collection tasks and
  resolver replay tasks through one declaration pass before replaying either
  side, covered by
  `cargo test --lib declaration_collection_replay_bundle_collects_ast_and_resolver_tasks_together`.
- Resolver validation replay tests now keep association-list replay coverage in
  `src/typechecker/tests/resolver_validation/replay_tasks/association_lists.rs`
  and declaration-task collector coverage in
  `src/typechecker/tests/resolver_validation/replay_tasks/declaration_tasks.rs`,
  reducing `src/typechecker/tests/resolver_validation/replay_tasks.rs` from 525
  lines to a module index while preserving coverage through
  `cargo test --lib resolver_validation::replay_tasks`.
- Build graph lowering now keeps build.zen DSL target kinds, field names, and
  builder method names in `src/build_graph/lowering/dsl.rs`, reducing
  `src/build_graph/lowering.rs` from 515 to 384 lines while preserving lowering
  behavior. `production_rust_files_stay_below_cleanup_threshold` now guards
  every tracked Rust source file against growing past the 500-line cleanup
  threshold.
- Build graph execution setup now shares one `BuildGraphExecutionContext`
  helper for graph loading, dependency gates, and base directory selection,
  with shared dependency ordering and existing non-executed source-validation
  ordering preserved, covered by `cargo test --test integration
  library_execution_gates`.
- Build graph execution dependency gates now validate reachable dependencies,
  not only direct dependencies from executed targets. Graph-only libraries can
  still be source/typechecked dependencies, but they cannot transitively pull in
  gated test targets during build execution or gated executable targets during
  test execution. Coverage includes
  `build_command_build_zen_rejects_transitive_gated_test_dependencies` and
  `test_command_build_zen_rejects_transitive_gated_executable_dependencies`.

## Unresolved Gaps

- Phase 2 is not complete: resolver/typechecker integration still has duplicate
  declaration collection for richer function type metadata and residual
  resolver-owned semantic handoffs.
- build.zen entrypoints are not complete: the constrained graph path now covers
  graph emission, multi-executable target execution for normal
  `zen build build.zen` and direct `zen build.zen`, dependency-ordered
  executable target execution for normal, direct, and legacy build graph
  commands, normal `zen test build.zen` test-target execution, normal
  `zen check build.zen` graph/source validation plus target source
  typechecking, library-only graph validation on the non-executing check path,
  graph-only library source validation and typechecking before build/test/emit
  execution, test and library target graph lowering/emission, single-target
  normal `zen emit build.zen`, validated library source dependencies for
  build/test/emit graph execution, direct and transitive gated executable/test
  dependency rejection for selected target kinds, library-only graph execution
  rejection, and targeted rejection for legacy generic `emit-json build.zen` modes. Library
  execution, package/link semantics, and other broader graph semantics still
  need explicit deterministic semantics before promotion.
- CLI compile helpers are now split into `src/cli/compile.rs`; this is an
  internal cleanup preserving the same build/emit behavior and keeping tracked
  Rust source files under the cleanup threshold.
- CLI JSON emit handlers are now split into `src/cli/json_emit.rs`,
  preserving frontend and build-graph JSON integration coverage while keeping
  JSON output paths separate from root command dispatch.
- Generic method-call checking now shares one helper for direct receiver
  methods and concrete generic receiver base methods, preserving existing
  generic method diagnostics and generated-C worklist coverage while reducing
  duplicate Phase 5 specialization logic.
- Generic function-call checking now shares one helper for direct calls and
  UFC calls, preserving generic function diagnostics, generic UFC fixture
  coverage, and generated-C specialization coverage while reducing duplicate
  Phase 5 specialization logic.
- Generic receiver base detection now uses the same generic concrete-name
  matcher as monomorphization, preserving generic method/generated-C
  specialization coverage while removing a hand-rolled mangled-name prefix
  check from method lookup.
- Generic direct-call and UFC-call deduplication for the same function
  instantiation is now covered by `tests/zen/generic_ufc_dedup.zen` and
  generated-C assertions that both calls resolve to a single emitted `id_i32`
  definition with no unspecialized `id` calls left behind.
- The removed source `return` keyword no longer leaves dead source or typed AST
  return-expression nodes behind. Final expressions remain the function result
  path, and `repo_hygiene::source_ast_no_longer_has_return_expression_nodes`
  guards the cleanup.
- Build graph host-effect lowering now treats wildcard fallback match arms as
  declared deterministic fallbacks for `b.os.env(...)` and
  `b.os.read_file(...)`, covered by build-graph lowering tests and executable
  `zen build build.zen` positive/negative fixtures.
- Build graph host-effect lowering also treats identifier fallback match arms
  as declared deterministic fallbacks, covered by env/file lowering tests and
  an executable `zen build build.zen` file-read fixture.
- Imported behavior association dependency seeding is now split into
  `src/typechecker/resolver_validation/imports_behavior_dependencies.rs`,
  preserving imported behavior inheritance and impl coverage while keeping the
  generic imported dependency include focused on callable/type seeding.
- Struct and enum aggregate expression construction is now split into
  `src/typechecker/expressions/aggregate_constructors.rs`, preserving focused
  struct literal, enum variant, and generic enum coverage while keeping
  member/array/index access support separate.
- Expected resolver type-parameter metadata helpers now live in
  `src/typechecker/resolver_validation_support/expected_type_parameters.rs`,
  preserving expected-symbol and resolver metadata validation descriptor
  coverage while keeping expected symbol constructors smaller.
- Expected resolver local expression, statement, and pattern traversal now lives
  in `src/typechecker/resolver_validation_support/expected_local_traversal.rs`,
  preserving resolver local replay coverage while keeping expected-local entry
  points smaller.
- Lexer string literal and interpolation tests now live in
  `src/lexer/tests/string_literals.rs`, preserving literal escape and
  interpolation coverage while keeping the parent lexer test module smaller.
- Lexer numeric literal and range-token tests now live in
  `src/lexer/tests/number_literals.rs`, preserving integer, float, prefixed
  integer, and float-vs-range token coverage while keeping the parent lexer
  test module smaller.
- Lexer whitespace and comment skipping now lives in `src/lexer/whitespace.rs`,
  preserving newline-token and string-interpolation comment coverage while
  keeping token dispatch smaller.
- Typed resolved type definitions and helpers now live in
  `src/ast/typed/types.rs`, preserving the public `ast::typed::Type` path while
  keeping typed AST node definitions smaller.
- Parser import declarations now live in `src/parser/import_declarations.rs`,
  preserving parser import coverage while keeping declaration dispatch smaller.
- Build graph check-source typechecking now lives in
  `src/cli/build_graph_execution.rs`, preserving `zen check build.zen` source
  diagnostics while keeping build graph validation helpers together.
- CLI module-graph loading and frontend typechecking now live in
  `src/cli/frontend.rs`, preserving check, emit, JSON, build, and run command
  frontend behavior while keeping root CLI dispatch smaller.
- CLI build graph loading and deterministic `build.zen` lowering now live in
  `src/cli/build_graph_loading.rs`, preserving build graph JSON/check behavior
  while keeping root CLI dispatch smaller.
- CLI build, test, direct-file, and legacy build-graph execution commands now
  live in `src/cli/execution_commands.rs`, preserving executable/test graph
  execution behavior while keeping root CLI dispatch smaller.
- CLI check and emit command bodies now live in
  `src/cli/check_emit_commands.rs`, preserving source validation and C emission
  behavior while keeping root CLI dispatch smaller.
- CLI build graph target structs and graph-target conversion helpers now live
  in `src/cli/build_graph_targets.rs`, preserving executable/test target
  behavior while keeping graph execution ordering and validation smaller.
- CLI build graph source existence and typechecking helpers now live in
  `src/cli/build_graph_sources.rs`, preserving source validation behavior while
  keeping graph execution ordering and dependency gating smaller.
- Build graph lowering target extraction now lives in
  `src/build_graph/lowering/targets.rs`, preserving deterministic target
  lowering coverage while keeping build-script expression traversal smaller.
- Build graph dependency validation tests now live in
  `tests/build_graph/dependencies.rs`, preserving unknown, self, cyclic, and
  dependency-order coverage while keeping the parent build-graph test target
  focused on target lowering and graph shape.
- Generic call-site, closure, and cast annotation diagnostics now live in
  `tests/generic_diagnostics/call_site_annotations.rs`, preserving malformed
  function/method type-argument annotation and cast/closure annotation coverage
  while keeping declaration/local annotation diagnostics focused.
- Range expressions now produce an explicit gated typechecker diagnostic,
  covered by parser and typechecker tests
  `parser::tests::expressions::parse_range_expr` and
  `typechecker::tests::core_semantics::literals::range_expression_is_rejected_until_range_type_exists`,
  so parser-accepted range syntax no longer succeeds with an unknown typed
  placeholder before range semantics exist.
- `Self` expression and statement validation traversal now lives in
  `src/typechecker/self_type_validation/expressions.rs`, preserving the same
  context diagnostics while keeping declaration task collection and type
  reference validation separate.
- Direct `zen emit build.zen` gated dependency validation tests now live in
  `tests/integration/cli_build/emit_direct_validation/gated_dependencies.rs`,
  preserving the gated dependency diagnostic coverage while keeping direct emit
  graph validation cases grouped by failure mode.
- Resolver-backed generic type-reference validation now lives in
  `src/typechecker/generic_type_validation/resolver_type_references.rs`,
  preserving resolver-restored callable/type metadata validation while keeping
  AST-owned type-reference validation separate.
- Parser expression suffix and aggregate literal parsing now lives in
  `src/parser/expressions/suffixes.rs`, preserving method/field access,
  struct literals, enum variants, and loop-control call parsing while keeping
  Pratt precedence dispatch separate.
- Resolver validation support type-info and generic template constructors now
  live in `src/typechecker/resolver_validation_support/type_info_constructors.rs`,
  preserving collected metadata construction while keeping method-key and AST
  type traversal helpers smaller.
- Resolver metadata diagnostic emitters now live in
  `src/typechecker/resolver_validation/metadata_diagnostics.rs`, preserving
  mismatch and absence diagnostics while keeping core resolver symbol
  requirement logic smaller.
- Parser atom expression forms now live in `src/parser/atoms/forms.rs`,
  preserving loop, match, cast, shorthand enum, and string interpolation
  parsing while keeping prefix atom dispatch smaller.
- Parser core state/navigation and Pratt precedence helpers now live in
  `src/parser/core.rs` and `src/parser/precedence.rs`, preserving parser unit
  coverage while keeping `src/parser/mod.rs` focused on module wiring and the
  public parse entry point.
- Resolver declaration validation dispatch now lives in
  `src/resolver/declaration_validation.rs`, preserving resolver Phase 2
  coverage while keeping `src/resolver.rs` focused on top-level orchestration.
- Resolver symbol-table definition builders now live in
  `src/resolver/symbol_table/definitions.rs`, preserving resolver Phase 2
  coverage while separating lookup methods from symbol construction.
- Resolver declaration replay-kind helpers now live in
  `src/typechecker/declaration_collection_resolver_tasks/replay_kinds.rs`,
  preserving type, behavior, impl, callable, and type-reference replay task
  construction while keeping declaration collection orchestration smaller.
- Resolver absence validation now keeps behavior, mutability, and source
  descriptor tables in
  `src/typechecker/resolver_validation_support/absence_symbol_descriptors.rs`,
  preserving absence diagnostics while keeping field and variant descriptors
  smaller.
- Error reporting now keeps `FileTable` and `FileId` source storage in
  `src/error/file_table.rs`, preserving the public `zen::error::FileTable`
  API while keeping `src/error.rs` focused on spans, diagnostics, and compile
  error conversion.
- Pattern checking now keeps enum and bool match validation in
  `src/typechecker/patterns/match_validation.rs`, preserving exhaustiveness,
  redundancy, and payload-shape diagnostics while keeping pattern binding and
  lowering smaller.
- Generic type reference validation now keeps expression and statement tree
  walking in `src/typechecker/generic_type_reference_walker/expressions.rs`,
  preserving generic annotation diagnostics while keeping type argument arity
  and bound checks smaller.
- Expression checking now keeps identifier, block, return, cast,
  interpolation, defer, and closure forms in
  `src/typechecker/expressions/simple_forms.rs`, preserving core semantic,
  single-file fixture, and generic diagnostic coverage while leaving the root
  expression checker closer to dispatch.
- C codegen now keeps runtime and stdlib stand-in helper emission in
  `src/codegen/c/types/runtime_helpers.rs`, preserving codegen unit coverage
  and runtime fixture execution while leaving C type/program/function emission
  smaller.
- Module-qualified calls now reject explicit type arguments on non-generic
  functions, covered by
  `module_function_explicit_type_args_are_error`, so `io.println<i32>(...)`
  cannot silently discard malformed type arguments.
- Typechecker method-call support now keeps module-qualified import call
  checking in `src/typechecker/expressions/method_call_support/module_calls.rs`,
  preserving module-call diagnostics and multi-file fixture coverage while
  keeping generic receiver/method/UFC resolution separate.
- Direct module-qualified calls now reject explicit type arguments on
  non-generic functions, covered by
  `builtin_function_explicit_type_args_are_error`, so
  `@builtin.panic<i32>(...)` cannot silently discard malformed type arguments.
- Generic callable behavior-bound diagnostics now live in
  `tests/generic_diagnostics/call_site_bounds.rs`, preserving function,
  method, receiver, Result method, unknown bound method, and UFC bound-failure
  coverage while keeping aggregate and annotation bound diagnostics focused.
- Generic behavior `requires` coverage now lives in
  `src/typechecker/tests/generic_behaviors/requires.rs`, preserving
  requires-pass, missing-impl, generic arity, missing type-argument, and
  nongeneric type-argument diagnostics while keeping behavior implementation
  tests focused.
- Resolver enum metadata validation tests now live in
  `src/typechecker/tests/resolver_struct_enum_metadata/enum_metadata.rs`,
  preserving variant payload count/type, visibility, typed payload,
  generic enum payload, variant-list, and owner diagnostics while keeping
  struct metadata validation focused.
- Resolver-restored behavior parent default synthesis tests now live in
  `src/typechecker/tests/resolver_collection/behavior_parents/default_synthesis.rs`,
  preserving inherited default method synthesis and generic default return
  substitution coverage while keeping parent restoration, duplicate, conflict,
  and cycle diagnostics focused.
- Generic behavior inheritance diagnostics now live in
  `src/typechecker/tests/generic_behaviors/extends/diagnostics.rs`,
  preserving parent/child overlap, inheritance cycle, duplicate parent,
  missing generic type arguments, nongeneric type arguments, inherited method
  conflict, and implementation signature mismatch coverage while keeping
  positive inheritance cases focused.
- Resolver expected-symbol leaf builder tests now live in
  `src/typechecker/tests/resolver_validation/expected_symbols/leaf_symbols.rs`,
  preserving import, module, local, behavior-edge, and behavior-association
  expectation coverage while keeping value, type, field, and variant
  expectation builders focused.
- Resolver-backed enum type metadata collection tests now live in
  `src/typechecker/tests/resolver_collection/type_metadata/enum_metadata.rs`,
  preserving enum payload/name restoration, stale AST enum rejection, and
  restored-name cleanup coverage while keeping struct field/default validation
  focused.
- Resolver-backed behavior impl default synthesis tests now live in
  `src/typechecker/tests/resolver_collection/behavior_impl_methods/default_synthesis.rs`,
  preserving behavior-name, impl-target, and combined restored metadata default
  synthesis coverage while keeping behavior impl method signature and impl-check
  metadata coverage focused.
- AST declaration validation task and replay tests now live in
  `src/typechecker/tests/declaration_validation/tasks.rs`, preserving self-type,
  precollection, declaration collection, type-reference, field default, and
  semantic replay task coverage while keeping direct validation diagnostics
  focused.
- Resolver-restored behavior method signature tests now live in
  `src/typechecker/tests/resolver_collection/behavior_methods/restored_signatures.rs`,
  preserving method name, return, parameter, order, and count restoration
  coverage while keeping incomplete-metadata and default-method behavior
  collection tests focused.
- Resolver behavior-association replay helper tests now live in
  `src/typechecker/tests/resolver_validation/replay_tasks/association_lists/association_validation.rs`,
  preserving extends, requires, combined association collection, and validation
  replay coverage while keeping resolver replay bundle aggregation tests
  focused.
- Extra resolver behavior-association metadata diagnostics now live in
  `src/typechecker/tests/resolver_behavior_impls_requires/extra_metadata.rs`,
  preserving extra impl/required name and ref mismatch coverage while keeping
  absent or wrong generic behavior association metadata tests focused.
- Resolver body/default local validation tests now live in
  `src/typechecker/tests/resolver_locals/body_locals.rs`, preserving pattern,
  top-level expression, closure, struct field default, and behavior default
  local coverage while keeping parameter and variable local metadata tests
  focused.
- Resolver-restored generic `Type.impl` method template tests now live in
  `src/typechecker/tests/resolver_collection/function_method_templates/type_impl_generic_methods/generic_templates.rs`,
  preserving template name, bound, return, parameter count, and mutability
  restoration coverage while keeping plain `Type.impl` method metadata tests
  focused.
- Resolver value absent declaration metadata tests now live in
  `src/typechecker/tests/resolver_value_metadata/absent_declaration_metadata.rs`,
  preserving rejected type, enum, behavior, parent, impl, and requires metadata
  on value symbols while keeping value signature and generic metadata
  validation focused.
- Resolver expected composite symbol builder tests now live in
  `src/typechecker/tests/resolver_validation/expected_symbols/composite_symbols.rs`,
  preserving behavior, struct, enum, and variant symbol metadata coverage while
  keeping primitive expected metadata and leaf symbol builders focused.
- Resolver behavior parent diagnostics now live in
  `src/typechecker/tests/resolver_collection/behavior_parents/diagnostics.rs`,
  preserving restored missing-method, inherited conflict, and cycle diagnostics
  while keeping behavior parent metadata restoration and default synthesis tests
  focused.
- Resolver-restored top-level generic method integrity tests now live in
  `src/typechecker/tests/resolver_collection/function_method_templates/generic_methods/integrity.rs`,
  preserving stale AST fallback rejection, restored-key cleanup, and body
  type-reference coverage while keeping template shape and mutability
  restoration tests focused.
- Resolver Phase 2 generic behavior method metadata tests now live in
  `tests/resolver_phase2/generic_behavior_metadata/method_signatures.rs`,
  preserving behavior method signature, function-typed method, duplicate
  parameter, and default-local coverage while keeping generic parameter and
  bound metadata tests focused.
- Direct `zen emit build.zen` graph-only library validation tests now live in
  `tests/integration/cli_build/emit_direct_validation/graph_only_libraries.rs`,
  preserving missing source, valid source, and library typecheck coverage while
  keeping executable target-count ambiguity checks focused.
- `zen build build.zen` graph-only library validation tests now live in
  `tests/integration/cli_build/build_command_validation/graph_only_libraries.rs`,
  preserving missing source, valid source, and library typecheck coverage while
  keeping executable-target and gated-dependency validation checks focused.
- Legacy `zen build-graph build.zen` graph-only library validation tests now
  live in
  `tests/integration/cli_build/legacy_graph_command_validation/graph_only_libraries.rs`,
  preserving missing source, valid source, and library typecheck coverage while
  keeping executable-target, gated-dependency, and missing-root validation
  checks focused.
- Imported behavior inheritance frontend diagnostics now live in
  `tests/integration/frontend_diagnostics/behavior_extends.rs`, preserving
  direct, imported-parent, and transitive parent-method diagnostics while
  keeping frontend helper and generic arity diagnostics focused.
- Generic explicit arity follow-up suppression diagnostics now live in
  `tests/generic_diagnostics/method_type_args/arity_followups.rs`, preserving
  function and method inference/argument follow-up checks while keeping direct
  method type-argument diagnostics focused.
- Lexer syntax example tests now live in
  `src/lexer/tests/syntax_examples.rs`, preserving Zen function/import,
  declaration, UFC, pattern-match, and method-call tokenization examples while
  keeping core token/operator coverage focused.
- C codegen type mapping tests now live in
  `src/codegen/c/tests/type_mapping.rs`, preserving primitive, string, pointer,
  named, and function-pointer type mapping coverage while keeping program
  generation and helper emission tests focused.
- C codegen helper tests now live in `src/codegen/c/tests/helpers.rs`,
  preserving identifier escaping, literal formatting, temporary naming, and
  simple statement emission coverage while keeping whole-program generation
  tests focused.
- C codegen whole-program generation tests now live in
  `src/codegen/c/tests/program_generation.rs`, preserving function, struct,
  enum, entry-point, payload enum, and defer generation coverage while keeping
  shared C codegen test fixtures focused.
- Resolver symbol table test-support setters for behavior metadata and
  aggregate metadata now live in
  `src/resolver/symbol_table_test_support/behavior_metadata.rs` and
  `src/resolver/symbol_table_test_support/aggregate_metadata.rs`, preserving
  existing test helper APIs while keeping the parent test-support module
  focused on shared lookup/indexing and generic value metadata setters.
- Single-file generic specialization generated-C assertions now keep enum
  specialization coverage in
  `tests/integration/generic_specializations/enum_generated_c.rs` and
  method/worklist specialization coverage in
  `tests/integration/generic_specializations/method_worklist_generated_c.rs`,
  preserving undefined-call and unspecialized-symbol checks while keeping the
  parent integration module focused on cross-fixture uniqueness.
- Multifile generic specialization generated-C assertions now keep imported
  enum dependency coverage in
  `tests/integration/generic_specializations/multifile_generated_c/enum_dependencies.rs`
  and imported method/worklist dependency coverage in
  `tests/integration/generic_specializations/multifile_generated_c/method_worklist_dependencies.rs`,
  preserving imported undefined-call and unspecialized-symbol checks while
  keeping the multifile generated-C module focused on submodule wiring.
- Generic inference conflict diagnostics now keep method and receiver conflict
  coverage in `tests/generic_diagnostics/inference_conflicts/methods.rs`,
  preserving direct, receiver-derived, Result enum, function-type, array,
  raw-pointer, and slice conflict checks while keeping the parent inference
  conflict module focused on generic function inference conflicts.
- Generic call-site bound diagnostics now keep method, receiver, Result enum,
  and UFC bound-failure coverage in
  `tests/generic_diagnostics/call_site_bounds/methods.rs`, preserving the
  no-followup-body-error checks while keeping the parent call-site bound module
  focused on generic function bound failures.
- Generic annotation arity diagnostics now keep local variable annotation
  coverage in `tests/generic_diagnostics/annotations/local.rs`, preserving
  struct/enum arity and unspecialized-generic checks while keeping the parent
  annotation module focused on function/signature annotations.
- Generic method type-argument diagnostics now keep `Option<T>` and
  `Result<T,E>` enum-method arity coverage in
  `tests/generic_diagnostics/method_type_args/enum_methods.rs`, preserving
  malformed enum-method arity and follow-up suppression checks while keeping
  the parent method type-argument module focused on direct method and
  non-generic callable cases.
- Build graph host-effect fallback recognition now keeps `Ok`/`Err` result
  variant spelling in `HostEffectResultVariant`, preserving the existing
  `.Err`, wildcard, and identifier fallback behavior while removing the raw
  string comparison from semantic lowering logic.
- Build graph target metadata lowering now validates duplicate and unknown
  target fields before graph construction, so malformed `build.zen` target
  metadata cannot silently disappear into a vague missing-target diagnostic.
  Coverage includes `build_program_lowering_rejects_duplicate_target_fields`,
  `build_program_lowering_rejects_unknown_target_fields`, and
  `emit_json_build_graph_rejects_unknown_target_fields`.
- Build graph target metadata extraction now validates required fields and
  field value shapes before graph construction, so missing `out_dir` and
  non-string string fields produce direct target-field diagnostics instead of
  falling through to missing graph targets. Coverage includes
  `build_program_lowering_rejects_missing_required_target_fields`,
  `build_program_lowering_rejects_invalid_target_field_types`, and
  `emit_json_build_graph_rejects_missing_required_target_fields`.
- Effect checking, typed allocator semantics, actors in std integration,
  JSON/YAML IR boundaries, and broader build graph execution remain gated by
  `docs/V1_SPEC.md`.

## Decision

Do not mark the objective complete. Continue Phase 2 work unless a later audit
shows the resolver/typechecker and build-gate requirements have concrete
coverage.
