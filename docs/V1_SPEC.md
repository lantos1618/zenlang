# Zen v1 Specification Draft

Status: v1 draft. This document is normative for intended v1 behavior, but the
feature matrix below controls what the rewrite compiler may currently advertise.

## Baseline

The active implementation is the rewrite compiler:

```text
source -> tokens -> AST -> module loader -> typechecker -> typed AST -> C backend -> cc
```

Compiler-owned semantic data is the source of truth. Serialized JSON and YAML are
interface formats only; they must be generated from or validated against checked
compiler data.

## Syntax Contract

Implemented syntax forms are limited to the forms covered by `tests/zen` and Rust
unit tests: declarations, function calls, local bindings, structs, enums, field
access, method-style calls, loops, `return`, `defer`, casts, string interpolation,
and pattern-style `?` arms supported by the parser and C backend.

Unsupported spec-like constructs must stay gated until parser and semantic tests
exist. This includes generic behavior association syntax such as
`Type.implements(Json<T>)` and `T: Json<T>`, plus unspecialized generic behavior
bounds such as `T: Json`, comptime execution, type matching, async operations,
actor syntax, package manifests, and `build.zen` execution.

## Accepted Syntax Forms

Every accepted syntax form must have a spec entry and Test Evidence before it is
advertised as implemented.

| Syntax form | Status | Test Evidence |
|---|---|---|
| Function declaration `name = (params) Return { ... }` | implemented | `parser::tests::parse_simple_function`, `tests/zen/functions.zen` |
| Method declaration `Type.method = (...) Return { ... }` | implemented | `parser::tests::parse_method`, `tests/zen/ufc.zen` |
| Struct declaration `Name: { field: Type }` | implemented | `parser::tests::parse_struct_def`, `tests/zen/structs.zen` |
| Enum declaration `Name: Variant, Payload(Type)` | implemented | `parser::tests::parse_enum_def`, `parser::tests::parse_enum_with_payload`, `tests/zen/enums.zen` |
| Local imports `{ name } = module.path` | implemented | `parser::tests::parse_import`, `module_system::tests::load_file_with_relative_import` |
| Immutable and mutable local bindings | implemented | `parser::tests::parse_immutable_var`, `parser::tests::parse_var_decl_mutable`, `tests/zen/mutability.zen` |
| `return`, `break`, `continue`, and `loop` forms used by fixtures | implemented | `parser::tests::parse_loop_expr`, `tests/zen/loops.zen` |
| Pattern-style `?` arms supported by parser/codegen | implemented | `parser::tests::parse_pattern_match`, `tests/zen/conditionals.zen`, `tests/zen/enum_match.zen` |
| Field access and struct literals | implemented | `parser::tests::parse_struct_literal`, `tests/zen/nested_structs.zen` |
| UFC-style method calls | implemented | `parser::tests::parse_ufc_chain`, `tests/zen/ufc.zen` |
| Cast expressions `cast(value, Type)` | implemented | `parser::tests::parse_cast_expr`, `tests/zen/cast.zen` |
| String literals and interpolation | implemented | `parser::tests::parse_string_interpolation`, `tests/zen/strings.zen` |
| Pointer and slice type syntax accepted by parser | implemented | `parser::tests::parse_pointer_types`, `parser::tests::parse_slice_type` |
| Generic syntax and explicit behavior bounds | experimental | `parser::tests::parse_nested_generics`, `parser::tests::parse_rejects_generic_behavior_function_bound_with_clear_error`, `parser::tests::parse_rejects_generic_behavior_type_bound_with_clear_error`, `typechecker::tests::generic_function_collection`, `typechecker::tests::generic_bound_rejects_unspecialized_generic_behavior`, `typechecker::tests::behavior_generic_bound_accepts_later_behavior_declaration`, `typechecker::tests::generic_behavior_bound_accepts_type_with_impl`, `typechecker::tests::generic_behavior_bound_accepts_inherited_behavior_impl`, `typechecker::tests::generic_behavior_bound_rejects_type_without_impl`, `tests/zen/behavior_inherited_generic_dispatch.zen` |
| Behavior declarations `Name: behavior { method: (Self) Return }` | experimental | `parser::tests::parse_behavior_declaration`, `typechecker::tests::behavior_declaration_collection` |
| Explicit behavior impl blocks `Type.implements(Behavior) { ... }` | experimental | `parser::tests::parse_behavior_impl_block`, `parser::tests::parse_rejects_generic_behavior_impl_with_clear_error`, `typechecker::tests::behavior_impl_with_required_method_passes`, `typechecker::tests::behavior_impl_missing_required_method_is_error`, `typechecker::tests::behavior_impl_can_omit_default_method`, `typechecker::tests::behavior_impl_overlapping_inherited_behavior_is_error`, `typechecker::tests::behavior_impl_generic_behavior_without_type_args_is_error`, `tests/zen/behavior_json_explicit_impl.zen` |
| Type association assertions `.requires` | experimental | `parser::tests::parse_behavior_requires_assertion`, `parser::tests::parse_rejects_generic_behavior_requires_with_clear_error`, `resolver_phase2::resolver_accepts_behavior_requires_known_type_and_behavior`, `typechecker::tests::behavior_requires_rejects_missing_impl`, `typechecker::tests::behavior_requires_generic_behavior_without_type_args_is_error` |
| Behavior inheritance `.extends` | experimental | `parser::tests::parse_behavior_extends_declaration`, `parser::tests::parse_rejects_generic_behavior_extends_with_clear_error`, `resolver_phase2::resolver_accepts_behavior_extends_known_behaviors`, `resolver_phase2::resolver_records_behavior_parent_names`, `typechecker::tests::behavior_extends_requires_parent_methods`, `typechecker::tests::behavior_extends_duplicate_parent_is_error`, `typechecker::tests::behavior_extends_generic_parent_without_type_args_is_error`, `typechecker::tests::behavior_extends_cycle_is_error`, `typechecker::tests::behavior_extends_conflicting_method_signature_is_error`, `tests/zen/behavior_inherited_default_method.zen` |

## Type, Module, ABI, Error, Effect, And Comptime Decisions

- `Sync/Async effects`: gated. `Sync` and `Async` are real effects in v1, not
  marker-only types. Sync code must not call async operations except through an
  explicit runtime blocking boundary. Async operations lower through checked task,
  queue, scheduler, yield, and await-like APIs.
- `Typed allocators`: gated. v1 allocators are typed by allocated value and effect
  mode, such as `Allocator<T, Sync>` and `Allocator<T, Async>`. Sync allocation
  returns a direct checked result; async allocation returns a task/effect result.
- `Type matching`: gated. Comptime type matching operates on typed metadata for
  primitives, structs, enums, fields, variants, behaviors, allocator modes, and
  effect modes. It is separate from runtime value matching.
- `Behavior association`: gated. Associated operations resolve by explicit impl,
  then generated impl, then declared fallback where the spec allows it. Ambiguity
  is a hard diagnostic.
- `AST traversal`: experimental. Raw AST traversal is for tooling and source
  transforms. Typed HIR traversal is required for semantic metaprogramming.
  Neither replaces compiler resolver, typechecker, effect checker, or MIR passes.
- `Actors in std`: gated. Actors are a stdlib framework first, with `Actor`,
  `ActorRef`, `Mailbox`, `Channel`, and `Supervisor` built on effect-aware queues
  and typed allocators. No actor syntax is v1-stable yet.
- `JSON/YAML IR boundaries`: gated. JSON is the machine-readable exchange format
  for compiler-owned AST, typed HIR, MIR, symbol tables, type layouts, and
  diagnostics. YAML is the human-authored format for target descriptions, ABI
  rules, intrinsic tables, allocator templates, backend options, and build graphs.
- `build.zen`: gated. `build.zen` executes under deterministic comptime APIs and
  builds a graph of targets, sources, dependencies, features, output dirs, and
  backend selections through target YAML plus compiler-owned IR outputs.
- Errors: `Result<T, E>` and `.raise()` are v1 design goals, but `.raise()` is
  gated until typechecked propagation and lowering are implemented.
- ABI: stable layouts for structs, enums, options/results, strings, slices,
  pointers, closures, and function pointers are gated until layout tests exist.

## Feature Matrix

| Feature | Status | Gate |
|---|---|---|
| Lexer/parser for tested fixtures | implemented | Existing unit and integration tests |
| Local module loading | implemented | Existing integration tests |
| Typechecked C backend for tested fixtures | implemented | `cargo test --tests` |
| README and contributor truth assertions | implemented | `tests/docs_truth.rs` |
| Strict resolver, symbol IDs, privacy | gated | Phase 2 tests |
| HIR/MIR JSON emission | gated | Schema and golden tests |
| Target/build YAML validation | gated | Schema and negative validation tests |
| Behaviors and type association | gated | Positive/negative behavior solver tests |
| `Sync/Async effects` | gated | Effect checker positive/negative tests |
| `Typed allocators` | gated | Sync and async allocator tests |
| Comptime type matching | gated | Type metadata and derive tests |
| Actors in std | gated | Mailbox, scheduling, supervisor tests |
| `build.zen` execution | gated | Deterministic build graph tests |
| Existing broad stdlib files | experimental | Must compile before promotion |
| Formatter, package manager, alternate backends | removed from v1 claims | Reintroduce only with tests and binaries |

## Required Test Backlog

Every v1 effect/type-match/allocator/actor/build-system claim needs at least one
planned positive test and one planned negative test before implementation.

| Area | Planned Positive Test | Planned Negative Test |
|---|---|---|
| `Sync/Async effects` | Async function may enqueue, yield, and call async operation through checked APIs | Sync function calling async operation without blocking boundary is rejected |
| `Typed allocators` | `Allocator<i32, Sync>` returns a checked pointer result and propagates into a container | `Allocator<i32, Sync>` cannot satisfy an `Allocator<i32, Async>` parameter |
| Type matching | `to_json<T>` derive branches on struct and enum metadata | Ambiguous or unreachable type-match arm is diagnosed |
| Behavior association | Explicit `Json<T>` impl takes precedence over generated derive fallback | Missing or ambiguous associated behavior impl is rejected |
| AST traversal | Tooling can read AST JSON for a parsed source fixture | AST traversal cannot bypass semantic checks for a core language feature |
| Actors in std | Actor mailbox send/receive works with scheduler and allocator integration | Actor using async mailbox from sync-only context is rejected |
| JSON/YAML IR boundaries | Checked MIR JSON and target YAML validate against schemas | Hand-authored JSON IR cannot override compiler-owned types or layouts |
| `build.zen` | Deterministic build graph creates one executable target | Build script using undeclared host side effects is rejected |

## Stdlib Gate

Files under `stdlib/` are experimental unless a test proves they parse, typecheck,
and build through the same compiler path as user modules. Aspirational stdlib
APIs must not be described as implemented until promoted by tests.
