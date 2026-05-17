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
- Nested generic enum specialization is covered by
  `tests/zen/generic_nested_result_enum.zen`, proving
  `Result<Option<i32>, str>` runtime behavior and generated-C call/definition
  consistency for both the outer result unwrap and inner option unwrap.
- Generic method specialization with nested generic enum returns is covered by
  `tests/zen/generic_method_nested_result.zen`, proving a specialized
  `Box<T>` method can infer `T` from the receiver and return
  `Result<Option<T>, str>` without leaving unspecialized C symbols or
  unresolved generated calls.
- Imported generic methods that return source-module generic enum dependencies
  now also rely on receiver inference in
  `tests/zen/multi_file_type_method_return_enum_dependency`, covering both the
  entry-module call and the source method's nested `self.wrap()` call.
- Imported generic methods returning nested source-module generic enum
  dependencies are covered by
  `tests/zen/multi_file_type_method_nested_result_dependency/main.zen`,
  proving `Result<Option<T>, str>` return specialization through an imported
  public method without undefined generated C calls.
- Imported generic `Result<T, E>` enum methods now cover multiple concrete
  instantiations from the importing module through
  `tests/zen/multi_file_generic_result_enum_multi_specialization/main.zen`,
  preserving `Result_unwrap_or_i32_str` and `Result_unwrap_or_bool_str`
  call/definition pairs without unspecialized `Result_T` symbols.
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
- Resolver-backed generic type-reference validation now has a focused
  resolver-backed traversal instead of interleaving resolver and AST-only
  validation in each declaration arm.
- Resolver-backed function type-reference validation now runs through a
  dedicated restored-function helper, reducing another inline resolver
  handoff inside the broad generic type-reference declaration scan.
- Resolver-backed top-level method type-reference validation now runs through a
  dedicated restored-method helper, aligning it with the restored-function
  type-reference path.
- Resolver-backed `Type.impl` method type-reference validation now shares the
  restored-method helper, so top-level and impl method body type-ref checks use
  the same resolver-owned key path.
- Resolver validation support now keeps behavior-specific AST type
  substitution, bound display, and method-signature comparison helpers in a
  focused include, separate from generic type-info construction helpers.
- Parser declaration dispatch now keeps struct, enum, and generic type
  parameter parsing in a focused declaration-types module, leaving the main
  declaration parser responsible for dispatch, imports, functions, and
  top-level bindings.
- Resolver symbol table data-model definitions now live in a focused core
  include, leaving symbol-table lookup and definition behavior in the parent
  implementation file.
- Typechecker declaration collection now keeps resolver replay task construction
  in a focused module, separate from declaration collection orchestration and
  resolver metadata application.
- Resolver validation now keeps expression, statement, and pattern local-scope
  traversal in a focused include, separate from resolver symbol/import and local
  symbol metadata validation.
- Resolver expression validation now keeps constructor-specific reference checks
  and scoped traversal helpers in a focused module, leaving the main expression
  validator as the dispatcher.
- Monomorphization now keeps generic struct/enum specialization and nested
  specialized type-ref emission in a focused module, separate from callable
  specialization and substitution helpers.
- AST declaration collection now keeps callable task dispatch and generic
  function/method template collection in a focused module, separate from import,
  impl-block, type, behavior, and precollection work.
- C codegen now keeps match, conditional, and controlled-loop lowering in a
  focused module, separate from statement and inline-expression emission.
- C codegen now keeps closure definition scanning in a focused module, separate
  from type, program, function, and global emission.
- Resolver collection tests now keep behavior default-method synthesis cases in
  a focused behavior-methods submodule, separate from signature metadata cases.
- Resolver collection tests now keep generic `Type.impl` method integrity cases
  in a focused submodule, separate from template shape and mutability metadata.
- Generic enum specialization coverage now includes a `Result<T, E>` enum method
  fixture with generated-C assertions for the concrete mangled method.
- Generic enum method specialization now also covers multiple concrete
  instantiations of the same `Option<T>` and `Result<T, E>` methods in one
  program through `generic_enum_multi_specialization` and
  `generic_result_enum_multi_specialization`, including generated-C definition
  uniqueness and call/definition assertions.
- Generic diagnostics now cover explicit type-argument arity failures for
  two-parameter `Result<T, E>` enum methods without noisy followup diagnostics.
- Imported generic enum methods now cover the same explicit type-argument arity
  failure through the module graph, preserving the hard method diagnostic
  without inference or argument-mismatch followups.
- Imported generic functions now cover the same explicit type-argument arity
  failure through the module graph, preserving the hard function diagnostic
  without inference or argument-mismatch followups.
- Generic diagnostics now cover generic enum constructor arity failures without
  leaking raw type-parameter payload mismatch followups, including
  `generic_enum_constructor_without_type_args_is_error`.
- Generic diagnostics now cover generic struct constructor arity failures
  without leaking raw type-parameter field mismatch followups, including
  `generic_struct_constructor_without_type_args_is_error`.
- Generic diagnostics now cover non-generic struct and enum constructors that
  receive type arguments, including
  `nongeneric_struct_constructor_type_args_are_error` and
  `nongeneric_enum_constructor_type_args_are_error`.
- Generic diagnostics now cover non-generic struct and enum annotations that
  receive type arguments, including
  `nongeneric_struct_annotation_type_args_are_error` and
  `nongeneric_enum_annotation_type_args_are_error`.
- Generic diagnostics now cover non-generic behavior references that receive
  type arguments across impls, requires, extends, and generic bounds, including
  `behavior_impl_nongeneric_behavior_type_args_are_error`,
  `behavior_requires_nongeneric_behavior_type_args_are_error`,
  `behavior_extends_nongeneric_parent_type_args_are_error`, and
  `generic_bound_nongeneric_behavior_type_args_are_error`.
- Generic diagnostics now cover receiver-vs-argument inference conflicts for
  two-parameter `Result<T, E>` enum methods.
- Generic diagnostics now cover bound failures on `Result<T, E>` enum methods
  without specializing failed method bodies into followup errors.
- Generic diagnostics now also guard plain generic function and method
  inference conflicts against argument/return mismatch followups.
- Generic enum method specialization coverage now includes an imported
  `Result<T, E>` fixture with generated-C assertions for the concrete method.
- Resolver-backed callable type-reference validation now shares the collected
  signature/body validation helper after function, top-level method, and
  `Type.impl` method dispatch restore the resolver-owned callable key, with
  focused coverage for stale callable names in body type refs.
- Resolver-backed top-level method collection also restores method names from
  resolver value symbols by declaration span when AST-only method names are
  stale, so collected `Type.method` signatures use resolver-owned names.
- Resolver-backed non-behavior `Type.impl` method collection now uses the same
  resolver-owned declaration-span handoff, so stale AST-only impl method names
  cannot leave stale `Type.missing` method entries during setup. Generic
  `Type.impl` method templates are covered by the same resolver-restored key,
  parameter, and return metadata path.
- Resolver-backed non-behavior `Type.impl` metadata collection now receives
  the impl target and method list from the declaration dispatcher instead of
  re-matching the whole declaration, shrinking another duplicate declaration
  collection boundary while preserving resolver-restored impl method coverage.
- Behavior impl method naming, resolver-owned key restoration, and
  behavior-specialized impl method symbol helpers now live in a focused
  typechecker module, keeping behavior association/default support below the
  previous size peak while preserving existing resolver-backed impl method
  coverage.
- Impl-block declaration collection now owns the `Type.impl` dispatcher for
  both AST and resolver-backed setup, so resolver-backed template stubs also
  receive the already-dispatched target and method list instead of re-matching
  the whole declaration.
- Callable declaration collection now owns the function/method dispatcher for
  both AST setup and resolver-backed template stubs, so callable collection
  shares one declaration walk before mode-specific signature or stub handling.
- AST type declaration collection now validates struct/enum generic bounds in
  the same dispatch that builds type metadata, removing the separate generic
  bound walk over type declarations.
- AST behavior declaration collection now queues behavior generic bounds while
  building behavior metadata, then validates them after all behavior names are
  collected without a separate behavior declaration walk.
- Behavior declaration collection now dispatches each behavior once and passes
  extracted signature fields into AST signatures or resolver-backed stubs,
  avoiding whole-list handoff inside behavior setup.
- AST behavior-extends validation now records explicit extends-validation
  tasks before replaying checks, keeping declaration filtering out of the
  validation pass while preserving cycle and coherence ordering.
- Struct field-default validation now dispatches struct declarations once and
  routes the extracted fields through AST or resolver-restored default checks,
  avoiding separate mode-specific declaration scans.
- AST callable type-reference validation now shares one signature/body helper
  across functions, top-level methods, and `Type.impl` methods while preserving
  each caller's existing return diagnostic span.
- `Self` type validation now shares one callable signature/body helper across
  functions, top-level methods, behavior default methods, and `Type.impl`
  methods while preserving each caller's existing `Self` allowance.
- `Self` type validation now also shares behavior-association type-argument
  validation across impl, requires, and extends declarations instead of walking
  each association form inline.
- Resolver-symbol validation now shares one callable local-symbol helper across
  functions, top-level methods, behavior default methods, and `Type.impl`
  methods while leaving declaration symbol checks at each call site.
- Resolver-symbol validation now shares generic behavior-association
  type-argument validation across impl, requires, and extends declarations,
  keeping unknown-tolerant resolver handoff checks on one path.
- Generic type-reference validation now shares strict and unknown-tolerant
  type-argument list walking across recursive type refs, expression type args,
  and resolver-owned behavior association refs.
- Expression checking now delegates aggregate/access forms through focused
  helpers for member access, struct literals, enum variants, array literals,
  and index access, keeping `check_expr` as dispatch while preserving existing
  diagnostics.
- Expression checking now delegates match, conditional, and loop lowering
  through focused control-flow helpers, keeping pattern binding, exhaustiveness,
  and loop-control lowering behavior unchanged while shrinking the dispatcher.
- Typechecker scope and import lookup methods now live in a dedicated
  scope-management helper module instead of the broad typechecker root.
- Typechecker resolver behavior-ref metadata collection and restoration now
  lives in a dedicated helper module, keeping resolver-owned impl/requires/
  extends handoff state out of the broad typechecker root.
- Typechecker resolver-backed declaration collection orchestration now lives in
  a dedicated helper module, keeping resolver handoff passes out of the broad
  typechecker root.
- Typechecker semantic validation now lives in a dedicated helper module,
  keeping behavior association validation and struct-field default checks out
  of the broad typechecker root.
- Typechecker AST-side declaration collection now lives in a dedicated helper
  module, keeping import, callable, type, behavior, and precollection task
  helpers out of resolver metadata collection.
- Typechecker generic behavior-bound validation now lives in a dedicated helper
  module, keeping bound declaration checks and substitution-based impl checks
  out of the broad generic type-reference traversal.
- Typechecker generic type-reference validation now delegates recursive type,
  expression, and statement walking to a dedicated helper module, keeping
  declaration task replay separate from AST traversal details.
- Typechecker monomorphization inference now lives in a dedicated helper
  module, keeping generic argument matching and conflict reporting separate
  from specialization emission and type substitution.
- Typechecker monomorphization type conversion and mangling helpers now live in
  a dedicated utility module, keeping symbol-safe mangle keys and Type/AstType
  round-tripping out of specialization emission.
- Resolver metadata construction helpers now live in a dedicated resolver
  module, keeping value signatures, type-parameter metadata, behavior refs, and
  method-key formatting out of the broad resolver traversal.
- Resolver declaration definition now lives in a dedicated resolver helper
  module, keeping top-level symbol registration separate from resolver
  validation replay.
- Resolver type-reference validation now lives in a dedicated resolver helper
  module, keeping parameter, type-parameter, and known-symbol checks separate
  from declaration and expression traversal.
- Resolver expression and local-symbol validation now live in dedicated
  resolver helper modules, keeping expression traversal, statement traversal,
  and scoped local binding separate from declaration validation.
- Resolver absent-metadata entry and diagnostic helpers now live in dedicated
  resolver validation support files, keeping shared absence diagnostics
  separate from per-symbol absence descriptor code tables.
- Resolver symbol-table test support now shares one test-only symbol lookup
  helper across metadata setters, dropping
  `src/resolver/symbol_table_test_support.rs` below the cleanup threshold while
  preserving resolver-backed metadata and Phase 2 resolver coverage.
- Resolver type/behavior metadata tests now keep behavior-method metadata cases
  in a focused submodule, dropping
  `src/typechecker/tests/resolver_type_behavior_metadata.rs` below the cleanup
  threshold while preserving the resolver-backed metadata checks.
- Resolver import metadata tests now keep module-symbol metadata cases in a
  focused submodule, dropping
  `src/typechecker/tests/resolver_import_metadata.rs` below the cleanup
  threshold while preserving import/module resolver metadata coverage.
- Module-system tests now keep module-graph loading cases in a focused
  submodule, dropping `src/module_system/tests.rs` below the cleanup threshold
  while preserving module graph/import coverage.
- Parser behavior declaration and impl-block parsing now live in a dedicated
  parser helper module, keeping behavior signatures and association syntax out
  of the broad top-level declaration dispatcher.
- Typechecker call signature and generic validation helpers now live in a
  dedicated expression helper module, keeping call-expression dispatch separate
  from argument, annotation, and return-flow validation.
- C backend unit tests now live beside the backend in a dedicated test module,
  keeping the production C backend entry point and identifier helpers compact.
- C backend expression-emission unit tests now live in their own test helper,
  keeping backend generation, C type mapping, and low-level expression emission
  coverage separate.
- Lexer unit tests now live beside the lexer implementation in a dedicated test
  module, keeping the public lexer API and character-span helpers compact.
- Parser block, closure, and argument-list helpers now live in a dedicated
  parser module, keeping atom parsing focused on prefix expression dispatch.
- Typechecker monomorphization template dependency install/restore now lives in
  a dedicated helper module, keeping generic specialization flow separate from
  temporary source-module dependency overlays.
- Generic diagnostic integration coverage now keeps nested/function/container
  annotation arity tests in a focused module, separate from direct generic
  call, method, local, and declaration annotation arity cases.
- Typechecker resolver struct/enum metadata tests now keep variant absence
  metadata coverage in a focused child module, separate from positive struct
  field and enum payload metadata mismatch cases.
- Resolver phase 2 generic behavior tests now keep association edge coverage
  in a focused integration module, separate from generic behavior declaration
  and method metadata capture.
- Typechecker declaration-validation tests now keep resolver replay task
  coverage in a focused child module, separate from AST precollection,
  Self-context, and type-reference validation cases.
- Typechecker resolver-local tests now keep absent local metadata coverage in a
  focused child module, separate from parameter, scoped binding, closure,
  pattern, and default-body local handoff cases.
- Generic-specialization integration tests now keep behavior-bound and
  imported-behavior generated-C checks in a focused child module, separate from
  direct generic enum, method, worklist, and multi-file type dependency checks.
- Module-system unit tests now live beside the module-system implementation in
  a dedicated test module, keeping module graph/load entry points compact.
- Resolver expected-local traversal now lives in a dedicated validation support
  helper, keeping expected metadata formatting separate from scoped local walks.
- Build graph lowering from parsed build.zen programs now lives beside the
  graph model, keeping graph validation separate from AST traversal details.
- Build graph lowering now owns accepted `build.zen` DSL spellings for target
  kinds, target fields, and builder identifiers in focused enums instead of
  scattering string literals through semantic lowering.
- Build graph lowering spelling tests now live in a focused lowering test
  module, keeping the production lowering implementation below the cleanup
  threshold while preserving the same spelling guards.
- Build-command integration tests now keep ordinary single-file build
  diagnostics, `build.zen` graph validation, and `build.zen` host-effect
  ordering in focused modules instead of one oversized command test file.
- `zen test build.zen` integration coverage now keeps test-target execution,
  deterministic host-effect rejection, and graph validation failures in
  focused modules instead of one oversized test-command file.
- `zen emit build.zen` integration coverage now keeps single-target C emission,
  deterministic host-effect rejection, and graph validation failures in
  focused modules instead of one broad emit-command test file.
- Direct `zen build.zen` integration coverage now keeps executable graph
  execution, deterministic host-effect rejection, and graph validation failures
  in focused modules instead of one oversized direct-command test file.
- Legacy `build-graph <build.zen>` integration coverage now keeps executable
  graph execution, deterministic host-effect rejection, and graph validation
  failures in focused modules instead of one threshold-adjacent command test
  file.
- Import-visibility integration coverage now keeps transitive dependency and
  private imported-method cases in focused modules, dropping
  `tests/integration/import_visibility.rs` below the cleanup threshold while
  preserving the same import visibility behavior checks.
- Resolver symbol-table behavior edge recording now lives in a focused helper,
  keeping symbol definition and lookup separate from association mutation.
- Typechecker resolver callable signature restoration now lives in a focused
  helper, keeping the main typechecker module below broad dispatcher size.
- Resolver absence validation for type parameters and value signatures now
  lives in a focused helper, separate from field/variant/behavior absence
  checks.
- Resolver-backed behavior impl method signature restoration tests now live in
  a focused resolver collection test module, keeping default/conformance tests
  separate from restored signature and generic-template coverage.
- Resolver validation replay now collects expected declaration symbols,
  expected local symbols, import-validation state, and behavior-association
  replay tasks in one declaration pass before checking resolver-owned extras,
  stripped resolver import metadata, and resolver behavior association lists.
  Behavior and non-behavior impl-block method expected-symbol collection now
  share the same helper inside that replay collector, and callable parameter/body
  local collection now shares one expected-local helper. Scoped expression local
  collection is also shared for struct field defaults and top-level
  expressions on both required and expected resolver-symbol replay paths.
  Closure parameter local collection now reuses the same parameter helper,
  preserving mutable parameter handoff metadata, and closure body local
  collection now shares one scoped helper. Child expression local collection is
  shared for loop, while, and conditional branch scopes.
  Match-arm pattern/body local collection also shares one scoped helper, and
  pattern binding insertion is shared for identifier and struct shorthand
  bindings. Variable-declaration local binding and the mutable handoff
  predicate are shared between required and expected statement replay. Block
  expression/statement local collection now shares one block-scope helper.
- Resolver-backed declaration metadata collection now builds callable, type,
  and behavior metadata tasks in one declaration dispatch before replaying the
  existing callable/type/behavior restoration order. Resolver-backed
  generic type-reference validation also routes through that same task
  collector instead of maintaining a second declaration match, and
  resolver-backed struct field default validation reuses the same collected
  type tasks on the fallback semantic path.
- Resolver-backed behavior impl metadata now builds restored impl-block tasks
  once and reuses them for both impl method signature restoration and omitted
  default-method synthesis. Impl-block declaration collection now also uses a
  named task collector for plain and behavior impl blocks.
- AST behavior-extends validation now uses a named validation task collector
  before parent, cycle, and method-coherence checks.
- Collected declaration semantic validation now records behavior impl checks
  with a named validation task collector, then replays behavior impl before
  requires validation. Behavior requires validation also uses a named task
  collector before replay.
- Resolver-backed type behavior-impl refresh now uses explicit restored type
  tasks instead of a callback traversal for the final association restoration
  pass.
- Resolver-backed `Type.impl` method type-reference validation now has a
  dedicated impl-method helper, keeping impl-block method filtering out of the
  broader resolver-backed type-reference declaration scan.
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
- Top-level generic method templates now have direct resolver-restored return
  presence and parameter-count coverage, matching the generic function template
  path.
  Resolver-restored top-level generic method parameter names also preserve
  positional mutability and ignore stale same-name AST parameter matches from
  different positions.
- Resolver-backed generic `Type.impl` method template collection now has the
  same function-typed parameter/return and behavior-bound metadata coverage as
  top-level generic method templates, so impl templates do not rely on stale
  AST-only generic signatures before monomorphization.
  It also directly covers resolver-restored return presence and parameter
  counts for generic impl method templates.
  Resolver-restored generic impl method parameter names also preserve
  positional mutability and ignore stale same-name AST parameter matches from
  different positions.
- Resolver-backed behavior method collection now rebuilds behavior parameters
  from resolver-owned parameter names and types, so stale AST-only missing or
  extra parameters cannot distort impl conformance checks.
  Direct coverage now includes stale AST behavior method parameter names and
  stale parameter ordering.
- Resolver-backed behavior method collection now also walks resolver-owned
  behavior method metadata in resolver order, so stale AST-only missing behavior
  methods cannot drop required methods from impl conformance checks.
- If resolver behavior-method metadata is incomplete and behavior collection is
  removed, default-body type-reference validation now skips that behavior
  instead of falling back to stale AST generic parameters.
- Resolver-backed behavior type-reference and default-body validation now runs
  through a dedicated restored-behavior helper, reducing another inline
  resolver handoff in the generic type-reference scan.
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
- Typechecker resolver expected behavior-symbol construction now pairs
  type-like expectations and behavior-method expectations through one
  expected-behavior constructor.
- Typechecker resolver expected struct-symbol construction now pairs type-like
  expectations and field expectations through one expected-struct
  constructor.
- Typechecker resolver expected enum-symbol construction now pairs type-like
  expectations and variant-name expectations through one expected-enum
  constructor.
- Typechecker resolver expected variant-symbol construction now pairs owner,
  visibility, and payload expectations through one expected-variant
  constructor.
- Typechecker resolver expected import-symbol construction now pairs import
  source expectations and default visibility through one expected-import
  constructor.
- Typechecker resolver expected module-symbol construction now pairs module
  name, absent source, and default visibility through one expected-module
  constructor.
- Typechecker resolver expected local-symbol construction now pairs local
  scope, mutability, absent source, and default visibility through one
  expected-local constructor.
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
- Typechecker resolver expected behavior-association aggregation now builds
  impl and required edge groups through one expected-association constructor.
- Typechecker resolver expected behavior-parent aggregation now builds
  `.extends` edge groups through one expected-edge collection constructor.
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
- Resolver-backed struct type-reference and field-default expression validation
  now runs through a dedicated restored-struct helper, reducing another inline
  resolver handoff in the generic type-reference scan.
- Resolver-backed enum type-reference validation now runs through a dedicated
  restored-enum helper, matching the restored struct type-reference path.
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
- Method-key method-name parsing is now shared by resolver-backed behavior impl
  conformance and behavior impl signature collection, removing duplicate
  `Type.method` prefix stripping in resolver-owned method-name selection.
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
- Resolver behavior-ref selection and behavior impl required-method restoration
  now share the same exact-match-then-front queue selector.
- Resolver behavior method metadata restoration now uses the same named queue
  selection family while preserving front AST methods needed by later resolver
  method metadata.
- Behavior impl conformance now resolves effective method names through a
  dedicated helper that shares resolver-owned name, AST-name, and collected
  signature fallback selection.
- Resolver-backed impl method-key restoration now has its own gated helper,
  keeping resolver-owned span lookup disabled during AST-only collection while
  feeding behavior impl conformance the restored key during resolver-backed
  collection.
- Resolver-backed behavior impl method signature collection now shares that
  restored impl-method key with a named required-method selection helper, so
  stale AST-only impl method names do not control which resolver value
  signatures are collected for conformance.
- Resolver-backed behavior impl conformance and default-method suppression now
  share one collected method-signature lookup helper.
- Impl method collection, resolver-backed impl restoration, default seeding,
  and resolver-backed method lookup now share one type-qualified method key
  helper.
- Resolver-backed method signature collection and generic type-reference
  validation now use the same type-qualified method key helper.
- Resolver symbol validation for top-level and impl method signatures now uses
  the shared type-qualified method key helper.
- Behavior impl conformance now uses the shared type-qualified method key
  helper before resolver-owned method name restoration.
- Resolver-backed behavior impl conformance now routes resolver-owned
  impl-method key selection through a dedicated gated helper before effective
  method-name selection.
- AST, resolver-backed, graph-import, dependency, and typed body method-key
  construction now route through the same type-qualified method key helper.
- Expression method lookup for module fallbacks, concrete receivers, and
  generic receiver bases now also uses that shared method-key helper.
- Resolver value-symbol definition for top-level and impl methods now also
  routes through a single type-qualified method key helper.
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
  direct conflict diagnostics. Generic method inference conflict coverage now
  mirrors those compound parameter shapes and includes slice parameters, so
  method receiver inference cannot hide later nested argument conflicts.
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
  It also uses resolver-owned impl method names when reporting extra methods,
  so stale AST-only required names cannot hide real resolver-owned extras.
  Direct coverage now also proves stale AST impl method parameter names and
  ordering are restored from resolver-owned value signatures before conformance.
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
  Cycle diagnostics also use restored resolver parent refs, so stale AST-only
  parent names cannot hide inherited cycles.
  Duplicate-parent checks also use restored generic parent refs, so stale
  AST-only parent type arguments cannot collapse distinct parent
  specializations.
  Inherited method-coherence diagnostics also use restored generic parent refs,
  so stale AST-only parent type arguments cannot hide conflicting inherited
  method signatures.
  Inherited missing-method diagnostics also use restored parent behavior refs
  when AST-only `.extends` parent names or type arguments are stale.
  Inherited behavior default synthesis also uses restored parent refs instead
  of stale AST-only `.extends` parent names, including restored generic parent
  type arguments.
- Resolver-backed `.requires` conformance checks now read validated resolver
  required-behavior refs, so stale AST-only required behavior type arguments
  cannot produce false missing-impl diagnostics.
  Distinct generic `.requires` specializations also stay resolver-owned when
  stale AST-only required type arguments collapse to the same specialization.
  Missing-impl diagnostics also use restored target type names, behavior names,
  and behavior type arguments when all AST-only `.requires` parts are stale.
  Restored `.requires` behavior refs also use inherited child behavior impls
  when checking whether the required parent behavior is satisfied.
- Resolver-backed `.implements` conformance checks now read validated resolver
  behavior impl refs before method conformance, so stale AST-only impl behavior
  type arguments cannot produce false method signature diagnostics.
  Overlap diagnostics also use restored generic impl refs, so stale AST-only
  impl type arguments cannot hide parent/child behavior conflicts.
  Duplicate-impl checks also use restored generic impl refs, so stale AST-only
  impl type arguments cannot collapse distinct behavior specializations.
  Missing-method diagnostics also use restored target type names, behavior
  names, and behavior type arguments when all AST-only `.implements` parts are
  stale.
- Resolver-backed `.implements` and `.requires` conformance now also falls back
  to declaration-order resolver refs when AST-only behavior names are stale, so
  validated resolver behavior associations cannot be shadowed by stale AST
  names during semantic checks.
  Explicit generic `.implements` collection now also directly covers stale AST
  target type names, behavior names, and behavior type arguments together.
- Resolver-backed behavior association validation now skips AST-only parent,
  impl, and required refs when resolver association metadata is missing, and
  clears stale impl associations before resolver-owned refresh.
- Resolver-backed declaration collection now separates resolver declaration
  metadata refresh, behavior impl metadata refresh, semantic validation, and
  final impl association refresh into named passes instead of one mixed
  collection loop.
  Callable resolver declaration metadata now has a focused traversal for
  functions, top-level methods, and non-behavior impl methods, and the
  function/method arms call the shared signature restoration helpers directly.
  Type resolver declaration metadata now has a focused traversal for structs
  and enums, with shared resolver-owned type-name and behavior-ref restoration
  before type-specific field or variant collection.
  Behavior resolver declaration metadata now has a focused traversal for
  behavior declarations that calls the shared behavior metadata collector
  directly.
  Behavior impl method signatures are now skipped by the generic declaration
  metadata refresh and owned by the behavior impl metadata pass, covered by
  `typechecker::tests::resolver_declaration_metadata_skips_behavior_impl_methods_until_behavior_impl_pass`.
  AST behavior declaration seeding, behavior generic-bound validation, and
  AST-only behavior inheritance validation now also have named helper passes.
  AST behavior signature seeding and resolver-backed behavior stub seeding are
  separate helper passes, keeping resolver placeholder collection out of the
  AST signature loop.
  Behavior inheritance validation now dispatches through a shared self-type
  context pass and an AST-only extends/coherence helper, keeping
  resolver-backed collection out of the extends/coherence traversal.
  Behavior declaration collection now dispatches to AST signature seeding plus
  behavior generic-bound validation, or resolver-backed stub seeding, avoiding
  duplicate AST-only diagnostics from the remaining collection loop.
  AST struct/enum generic-bound validation and type declaration seeding now
  also have named helper passes.
  Type declaration collection now dispatches to the AST-only generic-bound and
  declaration-seeding helpers as one path, instead of entering those helpers
  during resolver-backed collection and relying on per-helper guards.
  AST callable generic-bound validation, callable signature seeding, and
  resolver-backed callable template seeding now also have named helper passes.
  Callable collection now dispatches to exactly one of AST generic-bound
  validation plus signature seeding, or resolver-backed template seeding,
  instead of invoking guarded AST-only passes during resolver-backed
  collection.
  AST impl method/default seeding and resolver-backed impl template seeding now
  also have named helper passes.
  Impl-block collection now dispatches to exactly one of those passes instead
  of invoking both and relying on per-pass resolver-backed guards.
  AST import seeding now has a named helper pass, removing the residual mixed
  declaration collection loop.
  Test-facing resolver replay task views now delegate to the same bundled
  resolver declaration metadata collector as production replay instead of
  running separate declaration scans, covered by
  `resolver_type_declaration_metadata_tasks_collect_only_type_work`,
  `resolver_callable_declaration_metadata_tasks_collect_callable_work`,
  `resolver_behavior_impl_block_declaration_tasks_collect_only_behavior_impls`,
  and `resolver_type_reference_validation_tasks_collect_only_type_reference_work`.
  Resolver-backed function and method signature restoration now shares one
  callable key repair and generic-template rekey helper.
  Resolver-backed semantic validation and final type impl association refresh
  now each have focused helper boundaries matching those named passes.
  The declaration metadata refresh pass also routes callable signatures,
  type declarations, and behavior declarations through focused helpers instead
  of one mixed match arm.
  Behavior declaration metadata now has the same focused helper boundary as
  callable and type declaration metadata.
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
- Generic behavior inheritance now also substitutes concrete child behavior
  type arguments while satisfying inherited parent behavior requirements,
  covered by `tests/zen/behavior_generic_parent_type_arg_inheritance.zen`.
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
- Resolver-backed `.requires` semantic validation now restores the required
  type target through a dedicated declaration helper instead of keeping that
  resolver handoff inline in the broad declaration semantics loop.
- Resolver-backed behavior-impl semantic validation now restores the impl type
  target through a dedicated declaration helper instead of keeping that
  resolver handoff inline in the broad declaration semantics loop.
- Resolver now rejects duplicate method names inside local behavior
  declarations, covered by
  `resolver_phase2::resolver_rejects_duplicate_behavior_method_names`.
- Resolver now rejects duplicate parameter names in behavior method signatures
  before typechecker metadata collection, covered by
  `resolver_phase2::resolver_rejects_duplicate_signature_parameter_names`.
- Resolver now records mutable closure parameter locals and typechecker
  resolver-backed validation rejects closure parameter mutability drift, covered
  by `resolver_phase2::resolver_records_mutable_closure_parameter_locals` and
  `typechecker::tests::check_program_with_symbols_validates_resolver_closure_parameter_mutability`.
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
- Distinct generic behavior specializations on one concrete type now emit and
  dispatch through behavior-specialized impl method symbols, covered by
  `tests/zen/behavior_distinct_generic_specialization_dispatch.zen` and
  generated-C assertions for `Point_encode__Json_str` and
  `Point_encode__Json_i32`.
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
  Omitted defaults also use resolver-restored behavior method names, so stale
  AST-only behavior default names cannot synthesize stale method keys.
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
- Resolver-backed struct field default validation now uses a dedicated
  resolver-restored field-default helper instead of keeping restored struct
  lookup inline in the broad declaration semantics loop.
- AST-only and resolver-backed struct field default declaration traversal now
  have separate helper passes before the focused default validators run.
- Resolver-backed struct field defaults are now stored and validated under
  resolver-owned field names by position, so stale AST-only field names cannot
  skip default type checking.
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
  symbol lookup across parent, impl, and requires handoff paths, and
  resolver-backed impl/requires semantic checks pop restored refs through one
  role-selected helper, covered by focused unit coverage.
- Resolver-backed callable template and behavior method collection now share
  parameter restoration from resolver-owned names and typed metadata, covered
  by focused unit coverage.
- Resolver-backed callable template and behavior method collection now share
  resolver return-type restoration for `void` versus annotated returns.
- Resolver-backed enum collection now uses one helper to restore resolver-owned
  variant names and owner-scoped typed payload metadata.
- Resolver-backed struct collection now restores resolver-owned field names,
  typed field metadata, and field defaults through one helper.
- Resolver-backed behavior collection now restores resolver-owned method lists
  and AST default bodies through one metadata helper.
- Resolver-backed behavior parent collection now restores parent refs and
  computed behavior keys through one metadata helper.
- Resolver-backed type implementation collection now restores impl association
  keys from resolver behavior metadata through one helper.
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
- Resolver-backed generic function and method template collection now seeds
  body-only template stubs before resolver metadata restoration, preserving
  positional mutability and spans without carrying AST-only generic names,
  parameter types, or return annotations.
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
- AST `Type.impl` method signature collection now owns generic-bound
  validation directly, removing a dead resolver-backed branch from the helper
  after resolver-backed impl collection split into its own template pass.
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
- Resolver-backed behavior collection now seeds only behavior method/default
  stubs before resolver metadata restoration, so generic names and bounds come
  from resolver symbols instead of the initial AST collection pass.
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
- Generic behavior impl method template restoration now uses the shared
  resolver callable key repair path, covered by
  `typechecker::tests::collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata`
  and
  `typechecker::tests::collect_declarations_with_symbols_clears_stale_behavior_impl_generic_method_template_after_key_restore`,
  so stale AST target/name keys cannot survive restored or incomplete resolver
  value-signature metadata.
- Resolver-backed `.requires` validation now restores stale AST target names
  from unique missing required-ref metadata before skipping incomplete resolver
  handoff, avoiding false diagnostics from stale AST-only required refs.
- Resolver-backed `.implements` validation and omitted-default synthesis now
  restore stale AST target names from unique missing impl-ref metadata before
  skipping incomplete resolver handoff, avoiding stale AST-only impl refs and
  default methods.
- Resolver-backed behavior default synthesis now uses a named skip helper for
  incomplete resolver impl-ref handoff, keeping the resolver-backed guard out
  of the default collection body.
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
- Resolver-backed callable signature collection now reads resolver parameter
  names, parameter types, and return types through one complete-signature
  helper.
- Resolver-backed struct field collection now reads resolver field metadata
  through one dedicated helper before restoring fields and defaults.
- Resolver-backed enum variant collection now reads resolver variant-name
  metadata through one dedicated helper before restoring owner-scoped payloads.
- Resolver-backed behavior method collection now reads resolver method metadata
  through one dedicated helper before restoring method signatures and defaults.
- Resolver-backed declaration info now reads resolver type-parameter names and
  typed bound refs through one shared metadata helper.
- Resolver-backed generic template refresh now uses the same complete
  type-parameter metadata as callable info, so incomplete typed bound refs do
  not leave stale function or method template type parameters behind.
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
- Resolver-backed stale-name recovery now routes resolver symbol/span lookup and
  behavior-ref owner fallback through a dedicated resolver lookup helper module.
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
- Generic type substitution now covers mutable pointers, raw pointers, slices,
  arrays, and function signatures so Phase 5 specializations do not leave
  nested type parameters inside composite type shapes.
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
- Resolver absent-metadata validation now routes value signature, type
  parameter, field, variant, behavior association, behavior declaration, and
  mutability entry builders through one typed replay helper while preserving
  role-specific diagnostic wrappers.
- Resolver type-parameter and value-parameter validation now share one
  metadata-list comparison helper for names, display metadata, and typed
  metadata after each count check, preserving each diagnostic code and message.
- Resolver behavior-method validation now uses the same metadata-list
  comparison helper for display signatures and typed method metadata,
  preserving behavior-specific resolver diagnostics.
- Resolver field validation now uses the same metadata-list comparison helper
  for display field metadata and typed field metadata after the field-count
  check, preserving type-specific resolver diagnostics.
- Resolver variant payload validation now uses a shared optional-metadata
  comparison helper for display payload metadata and typed payload metadata
  after the payload-count check, preserving variant-specific resolver
  diagnostics.
- Resolver value return validation now uses the same optional-metadata
  comparison helper for display return metadata and typed return metadata,
  preserving value-specific resolver diagnostics.
- Resolver enum variant-name validation now uses the shared metadata-list
  comparison helper for resolver-owned variant names, preserving
  type-specific resolver diagnostics.
- Resolver enum variant owner-name validation now uses the shared
  optional-metadata comparison helper for resolver-owned owner names,
  preserving variant-specific resolver diagnostics.
- Resolver declaration metadata task collection now gathers callable, type,
  behavior, and behavior-impl block tasks in one declaration pass before
  replaying the existing resolver-backed collection order. Type behavior-ref
  refresh now reuses the collected type tasks, reducing another duplicate
  declaration scan.
- Resolver-backed behavior impl semantic validation now reuses the collected
  behavior-impl block tasks instead of rebuilding impl tasks from declarations
  during the semantic replay pass.
- Resolver-backed behavior requires semantic validation now reuses collected
  requires tasks from the declaration metadata pass instead of rebuilding
  requires tasks during semantic replay.
- Resolver-backed struct field default validation now reuses collected type
  metadata tasks instead of rescanning declarations during semantic replay.
- Resolver-backed generic type-reference validation now reuses type-reference
  tasks from the declaration metadata pass instead of rescanning declarations
  during semantic replay.
- Resolver behavior declaration metadata now uses a named metadata task instead
  of anonymous tuple fields, keeping behavior metadata replay aligned with the
  other resolver declaration task types.
- Behavior impl conformance now carries effective impl methods as named
  declaration/name records instead of positional tuples, keeping resolver-owned
  method-name replay explicit.
- Generic template dependency save/restore entries now use named `name` and
  `previous` fields instead of raw `(name, previous)` tuples in the
  monomorphization dependency snapshots.
- Callable declaration collection now uses a named task collector for
  top-level functions and methods before replaying the existing AST or
  resolver-backed collection path.
- AST struct and enum declaration collection now uses a named task collector
  before replaying the existing generic-bound validation and type registration
  path.
- Behavior declaration collection now uses a named task collector before
  replaying AST signature registration or resolver-backed behavior stubs.
- AST import declaration collection now uses a named task collector before
  seeding imported names into the typechecker import table.
- AST struct field-default validation now uses a named task collector before
  replaying default expression checks for nongeneric structs.
- AST generic type-reference validation now uses a named task collector before
  replaying declaration-specific type and expression reference checks.
- Self-type context validation now uses a named task collector before replaying
  declaration-specific `Self` allowance checks.
- Resolver validation replay now separates declaration replay collection from
  final behavior-association replay task construction.
- Resolver-backed type-reference validation now uses a narrow type-reference
  task collector instead of collecting the full resolver metadata task bundle
  when only type-reference replay is needed.
- Resolver-backed struct field-default validation now uses a narrow type
  declaration metadata task collector instead of collecting the full resolver
  metadata task bundle when only type default replay is needed.
- Resolver-backed behavior declaration metadata now uses a narrow behavior
  metadata task collector shared by the full resolver metadata collector.
- Resolver-backed callable declaration metadata now uses a narrow callable
  metadata task collector shared by the full resolver metadata collector.
- Resolver-backed behavior impl block metadata now uses a narrow impl-block
  task collector shared by the full resolver metadata collector.
- Resolver-backed behavior requires validation now shares one requires-task
  push helper between the dedicated requires collector and the full resolver
  metadata collector.
- Behavior impl validation now uses a shared impl-task push helper before
  replaying conformance checks.
- Behavior extends validation now uses a shared extends-task push helper before
  replaying parent-edge checks.
- AST struct field-default validation now uses a shared default-task push
  helper before replaying default expression checks.
- AST import declaration collection now uses a shared import-task push helper
  before seeding imported names.
- Impl-block declaration collection now uses a shared impl-block task push
  helper before replaying type and behavior impl setup.
- Callable declaration collection now uses a shared callable-task push helper
  before replaying function and method signature setup.
- AST type declaration collection now uses a shared type-task push helper
  before replaying generic-bound validation and type registration.
- Behavior declaration collection now uses a shared behavior-task push helper
  before replaying signature setup and behavior generic-bound validation.
- Self type context validation now uses a shared context-task push helper
  before replaying declaration and expression `Self` checks.
- AST type reference validation now uses a shared reference-task push helper
  before replaying generic type-reference diagnostics.
- Resolver validation replay now uses shared behavior-association and parent
  list push helpers before checking resolver metadata lists.
- Resolver validation replay now uses a shared callable-symbol helper for
  top-level functions, methods, and impl methods.
- Resolver validation replay now uses a shared association-source helper for
  type and behavior declaration symbols.
- Resolver validation replay now uses a shared import-symbol helper for
  expected module and import entries.
- Resolver validation replay now uses shared behavior-edge helpers for impl,
  requires, and parent edges.
- Resolver validation replay now uses a shared variant-symbol helper for
  expected enum variant entries.
- Resolver validation replay now uses a shared scoped-expression helper for
  struct field defaults and top-level expressions.
- Resolver-backed callable replay now shares one helper for callable metadata
  and callable type-reference validation tasks, so functions, top-level
  methods, and impl methods are classified once before their restored
  signature and body-reference replay paths run.
- Resolver-backed type declaration replay now shares one helper for struct/enum
  metadata and type-reference validation tasks, so type declarations are
  classified once before restored field, variant, and body-reference replay
  paths run.
- Resolver-backed behavior declaration replay now shares one helper for
  behavior metadata and type-reference validation tasks, so behavior
  declarations are classified once before restored method metadata and
  default-body reference replay paths run.
- Resolver-backed behavior impl-block replay now shares one helper for
  behavior-impl metadata and impl-block type-reference validation tasks, so
  behavior implementation blocks are classified once before restored impl
  metadata and method body-reference replay paths run.
- Behavior requires replay now uses one named helper for both standalone
  semantic validation task collection and resolver-backed declaration metadata
  collection, so requires declarations are classified consistently before
  behavior-association validation runs.
- Behavior extends replay now uses one named helper for parent-edge semantic
  validation task collection, so behavior inheritance declarations are
  classified consistently before parent validation, cycle checks, and method
  coherence run.
- Behavior impl validation now reuses the resolver behavior-impl declaration
  task shape and helper, so standalone validation and resolver metadata replay
  classify `.implements` blocks through the same path.
- Resolver-backed behavior impl validation now calls the shared behavior-impl
  validator directly, removing the resolver-only forwarding loop over the same
  `.implements` task data.
- Standalone behavior-association validation now collects `.implements` and
  `.requires` tasks in one declaration pass before replaying the shared
  validators.
- The same standalone behavior-association task collector now also carries
  `.extends` parent edges for the early inheritance validation pass, replacing
  the old extends-only collector while preserving validation order.
- Resolver declaration metadata tasks now store behavior edge work in the same
  behavior-association bundle shape, so resolver-backed `.implements` and
  `.requires` replay no longer uses separate top-level task fields.
- Standalone and resolver-backed behavior-association semantic replay now share
  one validator over the bundled tasks, while the underlying impl/required edge
  checks remain focused helpers.
- Resolver-backed behavior impl metadata collection now receives the bundled
  behavior-association tasks and selects `.implements` entries internally,
  keeping resolver call sites aligned around the shared task shape.
- Resolver-backed collected semantic validation now receives the full resolver
  declaration metadata task bundle and selects behavior, type-reference, and
  field-default work internally instead of taking parallel slices.
- Prefix loop syntax now accepts `loop((l) { ... })` with enum-backed
  `done`/`next` control actions, including nested outer-loop exits and UFC
  `done(l)` / `next(l)` forms, with fixture and docs coverage.
- Resolver-backed type behavior refresh now receives the full resolver
  declaration metadata task bundle and selects type declarations internally,
  keeping the final replay step aligned with the bundled call sites.
- Resolver-backed behavior impl metadata collection now receives the full
  resolver declaration metadata task bundle and selects behavior impl entries
  internally, so collected replay call sites all use the same bundle shape.
- Resolver-backed callable metadata collection now receives the full resolver
  declaration metadata task bundle and selects callable entries internally,
  continuing the resolver replay move away from parallel slices.
- Resolver-backed type declaration metadata collection now receives the full
  resolver declaration metadata task bundle and selects type entries
  internally, keeping declaration metadata replay helpers bundle-shaped.
- Resolver-backed behavior declaration metadata collection now receives the
  full resolver declaration metadata task bundle and selects behavior entries
  internally, completing the declaration metadata replay helper bundle shape.
- Resolver behavior impl-block restoration now receives the full resolver
  declaration metadata task bundle and selects `.implements` entries
  internally, keeping behavior impl metadata replay on the bundled task shape.
- Behavior association impl and requires validators now receive the full
  behavior-association task bundle and select their entries internally,
  shrinking the remaining slice handoffs in association validation replay.
- Resolver-backed struct field-default validation now receives the full
  resolver declaration metadata task bundle and selects type declarations
  internally, removing another `tasks.types` handoff from semantic replay.
- Resolver-backed type-reference validation now receives the full resolver
  declaration metadata task bundle and selects type-reference replay entries
  internally, keeping collected semantic replay on the bundled task shape.
- Behavior association semantic validation now accepts either the standalone
  behavior-association bundle or the full resolver declaration metadata bundle,
  so resolver semantic replay no longer passes a nested association slice.
- Resolver validation replay now passes the full replay task bundle into
  behavior association list validation, which selects type and parent
  association lists internally.
- Resolver extra declaration/local symbol validation now receives the full
  resolver validation replay task bundle and selects expected symbol sets
  internally, reducing another replay sub-bundle handoff.
- Stripped resolver import validation now receives the full resolver
  validation replay task bundle and reads the import-validation flag
  internally, completing the current resolver replay call-site bundle shape.
- Resolver behavior-association list replay now selects type and behavior
  association entries from the resolver declaration task bundle internally,
  reducing another declaration replay sub-slice handoff.
- AST behavior extends validation now receives the full behavior-association
  task bundle and selects `.extends` entries internally, removing another
  association sub-slice handoff.
- The main AST declaration collection path now builds behavior, type,
  callable, impl-block, and import collection task lists in one declaration
  pass before replaying them in the existing collection order.
- AST declaration semantic validation now builds behavior-association,
  type-reference, and struct field-default validation task lists in one
  declaration pass before replaying the existing validation order.
- AST declaration semantic validation now replays that full validation task
  bundle through one helper instead of unpacking sub-slices at the entrypoint.
- Resolver validation replay tests now keep association-list replay cases and
  declaration-task collector cases in focused submodules, leaving the parent
  replay test module as a small index.
- AST pre-collection validation now builds `Self`-context and behavior-extends
  validation tasks in one declaration pass before replaying the existing
  validation order.
- AST declaration collection now replays the full collection task bundle from
  one helper instead of fanning out sub-slices at the entrypoint, preserving
  behavior/type/callable/impl/import collection order.
- AST declaration collection now carries pre-collection validation tasks in
  the same declaration task bundle, so behavior declarations are collected and
  pre-collection validations are replayed without a second declaration scan.
- Resolver-backed declaration semantic validation now builds one resolver
  metadata task bundle and replays behavior-association, type-reference, and
  struct field-default validation from it.
- The first deterministic build-graph core is covered by
  `tests/build_graph.rs`: `deterministic_build_graph_creates_one_executable_target`
  proves canonical graph emission for an executable target, and
  `build_graph_rejects_undeclared_host_effects` proves undeclared host effects
  are rejected before any `build.zen` execution is promoted.
- The checked-in `examples/project/build.zen` syntax is covered by
  `parse_project_build_zen_example`, and leading-dot enum shorthand used by
  build-style result/config flows is covered by
  `parse_shorthand_enum_variant_expr_and_pattern`.
- A constrained `build.zen` lowering boundary now maps parsed build scripts into
  `BuildGraph` without enabling CLI execution. `parsed_project_build_zen_lowers_to_executable_and_test_graph`
  covers the checked-in project executable and test targets,
  `build_program_lowering_collects_test_target` covers default naming for
  `Test { root: ... }`, `build_program_lowering_collects_library_target`
  covers graph-only `Library { name: ..., exports: ... }` targets,
  `build_program_lowering_collects_target_dependencies_and_features` covers
  target metadata arrays,
  `build_program_lowering_rejects_self_target_dependencies` covers
  self-dependency rejection,
  `build_program_lowering_rejects_cyclic_target_dependencies` covers
  dependency-cycle rejection,
  `build_program_lowering_rejects_unknown_target_dependencies` covers
  unresolved target dependency rejection,
  `build_program_lowering_rejects_unsupported_package_targets` and
  `build_program_lowering_rejects_unsupported_link_targets` keep package/link
  target semantics gated with targeted diagnostics.
  `build_target_dsl_kind_owns_source_spelling` keeps accepted build target
  source spellings and the supported-target diagnostic list owned by the DSL
  kind enum, and `build_target_kind_owns_diagnostic_spelling` keeps runtime
  target-kind diagnostic spellings owned by the build graph target kind enum,
  avoiding duplicated magic strings in semantic/CLI logic.
  `build_program_lowering_rejects_undeclared_env_reads` keeps undeclared host
  effects rejected during lowering. The constrained deterministic-effect
  surface also recognizes declared `b.os.read_file("...")` effects, covered by
  `build_program_lowering_accepts_declared_file_reads`, while
  `build_program_lowering_rejects_undeclared_file_reads` keeps undeclared file
  reads rejected before graph promotion.
- `emit-json build-graph <build.zen>` now exposes the constrained graph-emission
  path without enabling normal build execution. `emit_json_build_graph_outputs_project_build_graph`
  covers the positive CLI path, `emit_json_build_graph_outputs_library_target`
  covers graph-only library target JSON emission,
  `emit_json_build_graph_outputs_target_dependencies_and_features` covers
  target metadata JSON emission, and
  `emit_json_build_graph_rejects_undeclared_host_effects` plus
  `emit_json_build_graph_rejects_undeclared_host_effects_before_test_target_lowering`
  and `emit_json_build_graph_rejects_undeclared_host_effects_before_library_target_lowering`
  and `emit_json_build_graph_rejects_undeclared_host_effects_before_target_metadata_lowering`
  cover negative host-effect paths through the advertised compiler command.
  `emit_json_build_graph_outputs_declared_file_read_effects` and
  `emit_json_build_graph_rejects_undeclared_file_read_effects` cover the
  matching positive and negative deterministic file-read effect pair through
  that same graph-emission command.
  `emit_json_build_graph_rejects_unknown_target_dependencies` covers unresolved
  target dependency rejection through the same graph-emission path, and
  `emit_json_build_graph_rejects_self_target_dependencies` covers
  self-dependency rejection there. `emit_json_build_graph_rejects_cyclic_target_dependencies`
  covers dependency-cycle rejection on that graph-emission path.
  `emit_json_build_graph_rejects_unsupported_package_targets` and
  `emit_json_build_graph_rejects_unsupported_link_targets` cover targeted
  package/link target rejection through the CLI graph-emission path.
  The same package/link target gate is covered before execution or emission by
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
- `build-graph <build.zen>` now consumes the deterministic graph for executable
  targets without widening the accepted `build.zen` subset.
  `build_graph_command_compiles_single_executable_target` covers the
  single-target positive path,
  `build_graph_command_compiles_multiple_executable_targets` covers
  multi-target execution,
  `build_graph_command_compiles_executable_dependencies_first` covers
  dependency-ordered execution, `build_graph_command_rejects_undeclared_host_effects`
  and `build_graph_command_multi_target_rejects_undeclared_host_effects` cover
  deterministic host-effect rejection before execution starts,
  `build_graph_command_rejects_graph_without_executable_targets` covers
  test-only graph rejection before execution starts,
  `build_graph_command_rejects_missing_graph_only_library_source` covers
  graph-only library export validation before execution starts,
  `build_graph_command_accepts_valid_graph_only_library_sources` covers valid
  graph-only library source validation before executable target execution,
  `build_graph_command_rejects_graph_only_library_type_errors` covers
  graph-only library typechecking before execution starts, and
  `build_graph_command_rejects_undeclared_host_effects_before_library_typechecking`
  preserves deterministic host-effect validation before graph-only library
  typechecking. `build_graph_command_rejects_gated_library_dependencies` and
  `build_graph_command_rejects_gated_test_dependencies` cover rejected
  executable dependencies on gated library/test targets, and
  `build_graph_command_rejects_missing_root_source` covers a target execution
  failure before normal `zen build build.zen` is ungated.
  Declared deterministic file-read effects are accepted through
  `build_graph_command_accepts_declared_file_read_effects`, while undeclared
  file reads reject before execution through
  `build_graph_command_rejects_undeclared_file_read_effects_before_execution`.
- Normal `zen build build.zen` now routes through the same constrained
  deterministic graph pipeline used by `build-graph <build.zen>`, covered by
  `build_command_routes_build_zen_through_deterministic_graph`. The normal
  build path now compiles multiple executable targets, covered by
  `build_command_build_zen_compiles_multiple_executable_targets`, and rejects
  test-only graphs before execution starts through
  `build_command_build_zen_rejects_graph_without_executable_targets`.
  Executable graph targets now compile dependencies before dependents, covered by
  `build_graph_orders_targets_before_dependents` and
  `build_command_build_zen_compiles_executable_dependencies_first`, and reject
  dependencies on gated library targets through
  `build_command_build_zen_rejects_gated_library_dependencies`. They also
  reject executable-target dependencies on gated test targets through
  `build_command_build_zen_rejects_gated_test_dependencies`. They reject
  graph-only library exports with missing sources through
  `build_command_build_zen_rejects_missing_graph_only_library_source`, accept
  valid graph-only library exports through
  `build_command_build_zen_accepts_valid_graph_only_library_sources`, and
  typecheck graph-only library exports before compiling executable targets
  through `build_command_build_zen_rejects_graph_only_library_type_errors`.
  They preserve host-effect validation before graph-only library typechecking
  through
  `build_command_build_zen_rejects_undeclared_host_effects_before_library_typechecking`,
  and reject
  undeclared host effects before dependency-ordered execution through
  `build_command_build_zen_rejects_undeclared_host_effects_before_dependency_execution`.
  The normal build path also rejects undeclared host effects before
  multi-target execution through
  `build_command_multi_target_build_zen_rejects_undeclared_host_effects`. The
  single-target rejection remains covered by
  `build_command_build_zen_rejects_undeclared_host_effects`.
  Declared deterministic file-read effects are accepted on the normal build
  path through `build_command_build_zen_accepts_declared_file_read_effects`,
  while undeclared file reads reject before target execution through
  `build_command_build_zen_rejects_undeclared_file_read_effects_before_execution`.
- Normal `zen check build.zen` validates the same constrained deterministic
  graph without compiling targets, covered by
  `check_command_validates_build_zen_graph`. It typechecks graph target
  sources without compiling them through
  `check_command_build_zen_typechecks_target_sources`. It rejects missing
  executable, test, and library sources through
  `check_command_build_zen_rejects_missing_executable_source`,
  `check_command_build_zen_rejects_missing_test_source`, and
  `check_command_build_zen_rejects_missing_library_source`, and rejects
  undeclared host effects before source validation through
  `check_command_build_zen_rejects_undeclared_host_effects_before_source_validation`.
  It also rejects undeclared host effects before target source typechecking
  through
  `check_command_build_zen_rejects_undeclared_host_effects_before_target_typechecking`.
  The single-target host-effect rejection remains covered by
  `check_command_build_zen_rejects_undeclared_host_effects`.
  Declared deterministic file-read effects are accepted through
  `check_command_build_zen_accepts_declared_file_read_effects`, while
  undeclared file reads reject before source validation through
  `check_command_build_zen_rejects_undeclared_file_read_effects_before_source_validation`.
  Library-only graphs remain valid on this non-executing path through
  `check_command_build_zen_accepts_library_only_graph_validation`.
- Normal `zen test build.zen` compiles and runs test graph targets, covered by
  `test_command_build_zen_runs_test_targets`, compiles and runs multiple test
  graph targets through `test_command_build_zen_runs_multiple_test_targets`,
  runs test target dependencies before dependents through
  `test_command_build_zen_runs_test_dependencies_first`,
  rejects executable-only graphs before execution starts through
  `test_command_build_zen_rejects_graph_without_test_targets`, and rejects
  undeclared host effects before test execution through
  `test_command_build_zen_rejects_undeclared_host_effects` and
  `test_command_multi_target_build_zen_rejects_undeclared_host_effects`.
  Test execution also rejects dependencies on gated library targets through
  `test_command_build_zen_rejects_gated_library_dependencies`, and rejects
  test-target dependencies on gated executable targets through
  `test_command_build_zen_rejects_gated_executable_dependencies`. It validates
  graph-only library exports before execution through
  `test_command_build_zen_rejects_missing_graph_only_library_source`,
  accepts valid graph-only library exports through
  `test_command_build_zen_accepts_valid_graph_only_library_sources`, and
  typechecks them before execution through
  `test_command_build_zen_rejects_graph_only_library_type_errors`. It also
  preserves host-effect validation before graph-only library typechecking
  through
  `test_command_build_zen_rejects_undeclared_host_effects_before_library_typechecking`.
  Declared deterministic file-read effects are accepted on the normal test
  path through `test_command_build_zen_accepts_declared_file_read_effects`,
  while undeclared file reads reject before test execution through
  `test_command_build_zen_rejects_undeclared_file_read_effects_before_execution`.
- Normal `zen emit build.zen` emits generated C for the single executable graph
  target without compiling a binary, covered by
  `emit_command_build_zen_outputs_target_c_source`, rejects ambiguous
  zero-target and multi-executable C emission through
  `emit_command_build_zen_rejects_graph_without_executable_targets` and
  `emit_command_build_zen_rejects_multiple_executable_targets`, and rejects
  undeclared host effects through
  `emit_command_build_zen_rejects_undeclared_host_effects`. It validates
  selected executable dependencies before emission through
  `emit_command_build_zen_rejects_gated_library_dependencies` and
  `emit_command_build_zen_rejects_gated_test_dependencies`, preserving gated
  library/test execution boundaries on the emit path. It validates
  graph-only library exports before emission through
  `emit_command_build_zen_rejects_missing_graph_only_library_source`,
  accepts valid graph-only library exports through
  `emit_command_build_zen_accepts_valid_graph_only_library_sources`, and
  typechecks them before emission through
  `emit_command_build_zen_rejects_graph_only_library_type_errors`. It also
  preserves host-effect validation before graph-only library typechecking
  through
  `emit_command_build_zen_rejects_undeclared_host_effects_before_library_typechecking`,
  while
  `emit_command_build_zen_reports_multi_target_ambiguity_before_missing_executable_source`
  and
  `emit_command_build_zen_reports_multi_target_ambiguity_before_graph_only_library_typechecking`
  keep multi-executable ambiguity ahead of per-executable and graph-only
  library source checks.
  Declared deterministic file-read effects are accepted through
  `emit_command_build_zen_accepts_declared_file_read_effects`, while
  undeclared file reads reject before C emission through
  `emit_command_build_zen_rejects_undeclared_file_read_effects`.
- Library-only build graphs remain non-executable across build, direct,
  legacy, emit, and test execution entrypoints through
  `build_command_build_zen_rejects_library_only_graph_execution`,
  `direct_file_command_build_zen_rejects_library_only_graph_execution`,
  `build_graph_command_rejects_library_only_graph_execution`,
  `emit_command_build_zen_rejects_library_only_graph_execution`, and
  `test_command_build_zen_rejects_library_only_graph_execution`.
- Direct `zen build.zen` now aliases the same constrained deterministic graph
  build path as `zen build build.zen`, covered by
  `direct_file_command_build_zen_routes_through_deterministic_graph`, and
  now compiles multiple executable targets through
  `direct_file_command_build_zen_compiles_multiple_executable_targets`. It
  compiles executable dependencies before dependents through
  `direct_file_command_build_zen_compiles_executable_dependencies_first`,
  rejects dependencies on gated library and test targets through
  `direct_file_command_build_zen_rejects_gated_library_dependencies` and
  `direct_file_command_build_zen_rejects_gated_test_dependencies`,
  rejects graph-only library exports with missing sources through
  `direct_file_command_build_zen_rejects_missing_graph_only_library_source`,
  accepts valid graph-only library exports through
  `direct_file_command_build_zen_accepts_valid_graph_only_library_sources`,
  typechecks graph-only library exports before execution through
  `direct_file_command_build_zen_rejects_graph_only_library_type_errors`,
  preserves host-effect validation before graph-only library typechecking
  through
  `direct_file_command_build_zen_rejects_undeclared_host_effects_before_library_typechecking`,
  rejects test-only graphs before execution starts through
  `direct_file_command_build_zen_rejects_graph_without_executable_targets`,
  and rejects undeclared host effects for single-target and multi-target graphs
  through
  `direct_file_command_multi_target_build_zen_rejects_undeclared_host_effects`
  and
  `direct_file_command_build_zen_rejects_undeclared_host_effects`.
  Declared deterministic file-read effects are accepted through
  `direct_file_command_build_zen_accepts_declared_file_read_effects`, while
  undeclared file reads reject before execution through
  `direct_file_command_build_zen_rejects_undeclared_file_read_effects_before_execution`.
- Build script lowering collects multiple executable targets deterministically,
  covered by `build_program_lowering_collects_multiple_executable_targets`.
- Legacy `emit-json ast|symbols|typed|diagnostics build.zen` modes stay
  rejected with a targeted `emit-json build-graph` diagnostic, covered by
  `legacy_emit_json_modes_reject_build_zen_with_graph_diagnostic`.
- Typechecker resolver metadata restoration helpers for struct fields, enum
  variants, behavior methods, callable params, and optional return types now
  live in `src/typechecker/resolver_metadata_collection.rs`, keeping
  `src/typechecker/mod.rs` focused on state and orchestration while preserving
  the existing resolver metadata restoration tests.
- CLI diagnostic rendering now lives in `src/cli/diagnostics.rs`, keeping the
  root `src/cli.rs` module focused on command dispatch and execution while
  preserving the existing CLI diagnostic integration tests.
- Resolver behavior-ref metadata validation now lives in
  `src/typechecker/resolver_validation/metadata_behavior_refs.rs`, keeping
  typed metadata validation separate from parent/impl/requires edge checks.
- Expression method-call checking now lives in
  `src/typechecker/expressions/method_call_support.rs`, keeping direct
  function-call checking separate from method, generic-method, and UFC
  resolution while preserving generic method fixture coverage.
- Expression function checking now lives in
  `src/typechecker/expressions/function_checking.rs`, keeping
  `src/typechecker/expressions.rs` focused on expression dispatch while
  preserving return/fallthrough and defer coverage.
- Build graph lowering DSL spellings and target-field ownership now live in
  `src/build_graph/lowering/dsl.rs`, reducing
  `src/build_graph/lowering.rs` below the 500-line cleanup threshold while
  preserving the existing build.zen lowering behavior. The guard
  `production_rust_files_stay_below_cleanup_threshold` keeps the focused
  production files from silently growing back past the cleanup threshold.

## Current Phase

Continue the smallest behavior-association and resolver/typechecker hardening
slices. Phase 3 C codegen is sufficient for the current tested fixtures, but
Phase 4 build-driver work still has constrained `build.zen` semantics: normal
`zen build build.zen` and direct `zen build.zen` execute multiple executable
  graph targets, `zen test build.zen` executes multiple test graph targets,
  `zen check build.zen` validates executable, test, and library targets in the
  graph, accepts library-only validation, and typechecks target sources without
  compiling them, `zen emit build.zen`
  emits target C for a single executable graph target, and build/test/emit
  validate and typecheck graph-only library target sources while build/test/emit
  execution rejects dependencies on gated non-selected target kinds and library
  execution remains explicitly gated by library-only graph rejection coverage.
  Legacy generic JSON emitters reject
  `build.zen` and point to
  `emit-json build-graph`.

Do not promote gated v1 features until the relevant positive and negative tests
exist and pass through the same compiler path advertised in `docs/V1_SPEC.md`.

## Next Small Slice

Continue the next smallest build-driver slice by expanding graph execution only
when a new positive graph fixture and a matching negative deterministic-effect
fixture exist first. Preserve the constrained accepted `build.zen` subset until
multi-target or richer build-script semantics are specified and tested.
