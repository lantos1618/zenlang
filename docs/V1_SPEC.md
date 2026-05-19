# Zen v1 Specification Draft

Status: v1 draft. This document is normative for intended v1 behavior, but the
feature matrix below controls what the rewrite compiler may currently advertise.
Exhaustive proof belongs in tests, golden fixtures, and git history; this spec
keeps representative Test Evidence only.

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
unit/integration tests: declarations, calls, imports, bindings, structs, enums,
field access, method-style calls, final-expression results, prefix loops, `defer`,
casts, string interpolation, and parser/codegen-supported `?` arms.

Unsupported spec-like constructs must stay gated until parser, resolver,
typechecker, codegen, diagnostics, JSON, and public examples agree on the shape.
This includes unspecialized generic behavior targets, comptime execution, type
matching, async operations, actor syntax, package manifests, and `build.zen`
execution beyond the constrained deterministic graph surface.

Developer UX and Agent UX are product requirements, not polish. The v1 language
surface should grow toward MoonBit-style toolchain integration, but the compiler
must not advertise unsupported language-server binaries or editor features as
implemented. The current contract:

- the VS Code extension remains a constrained editor wrapper until language
  server tests exist;
- `zen lsp` remains gated until it is backed by the same parser, resolver,
  typechecker, build graph, and diagnostics as the CLI;
- Agent-readable diagnostics keep stable codes, spans, related locations,
  suggested fixes, feature_gate metadata, and JSON output aligned with CLI and
  editor behavior;
- the machine-readable project graph and symbol graph remain compiler-owned
  outputs for modules, imports, visibility, targets, dependencies, generated
  symbols, examples, and stdlib gates;
- structured fix suggestions are planned for missing match arms, generic arity
  mistakes, removed syntax such as `return`, gated features, missing imports, and
  common type mismatches;
- quiet deterministic commands such as `zen check`, `zen test`, and
  `zen emit-json` are required for agents and editors before automated fix or
  package workflows can be promoted.

## Accepted Syntax Forms

Every accepted syntax form must have a spec entry and Test Evidence before it is
advertised as implemented.

| Syntax form | Status | Test Evidence |
|---|---|---|
| Function declaration `name = (params) Return { ... }` | implemented | `parser::tests::parse_simple_function`, `tests/zen/functions.zen` |
| Method declaration `Type.method = (...) Return { ... }` | implemented | `resolver_records_method_signatures_as_value_symbols`, `resolver_records_method_function_type_signatures`, `check_program_with_symbols_validates_resolver_method_signature`, `tests/zen/multi_file_type_method/main.zen` |
| Non-behavior impl blocks `Type.impl = { ... }` and `Type<T>.impl = { ... }` | experimental | `parse_impl_block`, `parse_generic_impl_block_hoists_receiver_type_params_to_methods`, `resolver_accepts_non_behavior_impl_blocks_as_method_symbols`, `tests/zen/generic_type_impl_methods.zen` |
| Struct declaration `Name: { field: Type }` | implemented | `resolver_records_struct_function_type_fields`, `resolver_rejects_duplicate_struct_field_names`, `resolver_rejects_unknown_struct_literal_types`, `tests/zen/structs.zen` |
| Enum declaration `Name: Variant, Payload(Type)` | implemented | `resolver_records_enum_function_type_payloads`, `resolver_records_generic_enum_function_type_payloads`, `tests/zen/enums.zen`, `tests/zen/duplicate_enum_variant_names.zen` |
| Local imports `{ name } = module.path` | implemented | `check_module_graph_entry_seeds_imported_function_type_signatures`, `check_module_graph_entry_specializes_imported_generic_functions`, `tests/zen/multi_file_generic/main.zen` |
| Immutable and mutable local bindings | implemented | `resolver_records_top_level_expr_locals`, `resolver_records_closure_locals`, `resolver_records_mutable_closure_parameter_locals`, `resolver_records_same_name_locals_in_distinct_scopes`, `tests/zen/mutability.zen` |
| Final expression results, `break`, `continue`, and prefix `loop((l) { ... })` controls | implemented | `parser::tests::parse_return_keyword_is_removed`, `parser::tests::parse_loop_control_param_expr`, `tests/zen/loops.zen` |
| Pattern-style `?` arms | implemented | `resolver_records_pattern_locals`, `check_program_with_symbols_requires_resolver_pattern_locals`, `tests/zen/conditionals.zen` |
| Field access and struct literals | implemented | `parser::tests::parse_struct_literal`, `tests/zen/nested_structs.zen` |
| UFC-style method calls | implemented | `parser::tests::parse_ufc_chain`, `tests/zen/ufc.zen` |
| Cast expressions `cast(value, Type)` | implemented | `parser::tests::parse_cast_expr`, `tests/zen/cast.zen` |
| String literals as baked `StaticString`; interpolation as non-owning `StaticString` views | implemented | `parser::tests::parse_string_interpolation`, `tests/zen/strings.zen`, `dynamic_string_type_is_rejected_as_allocator_backed_gate` |
| Pointer and slice type syntax accepted by parser | implemented | `parser::tests::parse_pointer_types`, `parser::tests::parse_slice_type` |
| Generic specialization for functions, structs, enums, and methods | implemented | `generic_specializations_emit_each_generated_c_definition_once`, `generic_specializations::enum_generated_c::enum_specializations_do_not_emit_unspecialized_c_symbols`, `tests/zen/generic_method_worklist.zen` |
| Explicit behavior association proving ground | implemented | `Type.implements(Behavior)`, non-generic explicit behavior associations, `tests/zen/behavior_json_explicit_impl.zen`, `tests/zen/behavior_json_generic_association.zen` |
| Generic syntax and explicit behavior bounds | experimental | `resolver_records_value_symbol_generic_bounds`, `resolver_records_value_symbol_generic_parameter_counts`, `check_program_with_symbols_validates_resolver_function_type_parameter_bound_refs`, `behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied`, `behavior_extends_generic_parent_accepts_child_type_parameter_arg` |
| Behavior declarations `Name: behavior { method: (Self) Return }` | experimental | `parse_public_behavior_declaration`, `resolver_records_public_visibility_for_exported_declarations`, `resolver_records_behavior_function_type_method_signatures` |
| Explicit behavior impl blocks `Type.implements(Behavior) { ... }` | experimental | `resolver_rejects_duplicate_behavior_impl_edges`, `resolver_records_behavior_impl_methods_as_value_symbols`, `resolver_records_behavior_impl_method_body_locals`, `imported_private_behavior_impl_methods_are_not_directly_visible` |
| Type association assertions `.requires` | experimental | `resolver_rejects_duplicate_behavior_required_edges`, `check_program_with_symbols_validates_resolver_generic_behavior_required_refs`, `tests/zen/multi_file_imported_behavior_requires/main.zen` |
| Behavior inheritance `.extends` | experimental | `resolver_rejects_duplicate_behavior_parent_edges`, `imported_behavior_extends_requires_parent_methods`, `imported_behavior_extends_imported_parent_requires_parent_methods`, `imported_behavior_extends_requires_transitive_parent_methods` |

## Type, Module, ABI, Error, Effect, And Comptime Decisions

- `StaticString` is baked into the program. It denotes literal/static text with
  stable storage and compile-time length in the generated runtime layout.
  The allocator-backed `String` type is owned, dynamic text and carries allocator
  identity before it can be promoted. String literals do not implicitly allocate
  or coerce into `String`; dynamic `String` construction must use an explicit
  allocator-aware path once promoted. String interpolation currently returns a
  `StaticString`-shaped non-owning view, but only literal text is guaranteed to
  be baked program storage; interpolation must not imply allocator-backed
  `String` construction. Source-level `String` use currently reports a gated
  allocator-backed text diagnostic, including the generic nested case pinned by
  `emit_json_diagnostics_generic_dynamic_string_gate_schema_matches_golden`.
- `Sync/Async effects`: gated. Sync code must not call async operations except
  through an explicit runtime blocking boundary. Async operations lower through
  checked task, queue, scheduler, yield, and await-like APIs. `async task enqueue`
  and `async yield` builtins are gated. Evidence anchors:
  `async_scheduler_intrinsics_are_rejected_as_gated_not_unknown`,
  `stdlib_async_runtime_import_is_gated_before_loading_sketch`,
  `module_graph_gates_stdlib_async_runtime_import_before_loading_sketch`,
  `emit_json_diagnostics_async_runtime_import_gate_schema_matches_golden`,
  `stdlib_sync_runtime_import_is_gated_before_loading_sketch`,
  `module_graph_gates_stdlib_sync_runtime_import_before_loading_sketch`,
  `emit_json_diagnostics_sync_runtime_import_gate_schema_matches_golden`,
  `atomic_intrinsics_are_rejected_as_effect_gates`, `@builtin.atomic_load`,
  `@builtin.atomic_store`, `@builtin.atomic_add`, `@builtin.atomic_sub`,
  `@builtin.atomic_cas`, `@builtin.atomic_xchg`, and `@builtin.fence`.
- `Typed allocators`: gated. `Allocator<T, Sync>` and `Allocator<T, Async>` are
  distinct typed allocator modes. Raw allocation and byte-memory intrinsics are
  gated until ownership and effect semantics exist. Evidence anchors:
  `raw_memory_intrinsics_are_rejected_as_allocator_gates`,
  `byte_memory_intrinsics_are_rejected_as_allocator_gates`,
  `sync_and_async_typed_allocator_modes_are_rejected_as_gated_not_unknown`,
  `stdlib_allocator_import_is_gated_before_loading_sketch`,
  `module_graph_gates_stdlib_allocator_import_before_loading_sketch`,
  `emit_json_diagnostics_allocator_import_gate_schema_matches_golden`,
  `@builtin.raw_allocate`, `@builtin.raw_deallocate`,
  `@builtin.raw_reallocate`, `@builtin.memcpy`, `@builtin.memmove`,
  `@builtin.memset`, and `@builtin.memcmp`.
- `Type matching`: gated. comptime type matching operates on typed metadata for
  primitives, structs, enums, fields, variants, behaviors, allocator modes, and
  effect modes. It is separate from runtime value matching and remains gated until typed metadata and derive lowering exist. Evidence anchors:
  `comptime_type_match_intrinsic_is_rejected_as_gated_not_unknown` and
  `primitive_and_enum_type_match_intrinsics_are_rejected_as_gated_not_unknown`.
- `Behavior association`: gated beyond the explicit proving ground. Associated
  operations resolve by explicit impl, then generated impl, then declared
  fallback where the spec allows it. Ambiguity is a hard diagnostic.
- `AST traversal`: experimental tooling. `zen emit-json ast <file>` emits
  `semantic_status: "unchecked"`; semantic acceptance must use typed JSON,
  diagnostics, check, build, or test paths.
- `Actors in std`: gated. Promoted actor framework spellings `Actor`,
  `ActorRef`, `Mailbox`, and `Supervisor` depend on effect-aware queues and typed
  allocators. `Channel` remains an experimental stdlib channel sketch, not a
  global actor builtin. Evidence anchors:
  `bare_actor_framework_types_are_rejected_as_gated_not_unknown`,
  `stdlib_actor_framework_import_is_gated_before_loading_sketch`,
  `module_graph_gates_stdlib_actor_framework_import_before_loading_sketch`, and
  `emit_json_diagnostics_actor_import_gate_schema_matches_golden`.
- `Ownership and raw pointer operations`: gated. Evidence anchors:
  `raw_pointer_intrinsics_are_rejected_as_ownership_gates`, `@builtin.gep`,
  `@builtin.gep_struct`, `@builtin.raw_ptr_cast`, `@builtin.ptr_to_int`,
  `@builtin.int_to_ptr`, `@builtin.load<T>`, and `@builtin.store<T>`.
- `Host syscalls`: gated until explicit host effect declarations and syscall ABI
  semantics exist. Evidence anchors:
  `syscall_intrinsics_are_rejected_as_host_effect_gates`, `@builtin.syscall0`,
  and `@builtin.syscall6`.
- Errors: `Result<T, E>` and `.raise()` are v1 design goals, but `.raise()` is
  gated until typechecked propagation and lowering are implemented.
- ABI: stable layout JSON exists for primitives, baked `StaticString`, pointers,
  slices, arrays, structs, and simple enums. Full options/results, closures, and
  function pointer ABI compatibility remain gated until broader layout tests
  exist.

## JSON/YAML IR Boundaries

JSON/YAML IR boundaries are constrained. JSON is the machine-readable exchange
format for compiler-owned AST, symbols, typed programs, diagnostics, HIR, MIR,
layout, and deterministic build graphs. YAML is the human-authored format for
target/build input.

Current commands: `zen emit-json ast <file>`, `zen emit-json symbols <file>`,
`zen emit-json typed <file>`, `zen emit-json diagnostics <file>`,
`zen emit-json hir <file>`, `zen emit-json mir <file>`,
`zen emit-json layout <file>`, `zen emit-json build-graph <file>`, and
`zen emit-json target-yaml <file>`.

Schema status: AST JSON is unchecked; symbols JSON is resolved; typed JSON is explicitly marked checked; diagnostics JSON is explicitly marked diagnostic. All schemas use `schema_version: 0` until promoted.

Representative golden anchors:

- hand-authored IR rejection:
  `emit_json_ast_rejects_hand_authored_json_before_unchecked_ir_override`,
  `emit_json_symbols_rejects_hand_authored_json_before_resolver_override`,
  `emit_json_typed_rejects_hand_authored_json_before_checked_ir_override`,
  `emit_json_diagnostics_rejects_hand_authored_json_before_diagnostic_override`,
  `emit_json_hir_rejects_hand_authored_json_before_ir_override`,
  `emit_json_mir_rejects_hand_authored_json_before_ir_override`,
  `emit_json_layout_rejects_hand_authored_json_before_layout_override`, and
  `emit_json_build_graph_rejects_hand_authored_json_before_graph_override`.
- symbols/typed/HIR/MIR/layout generics:
  `emit_json_ast_module_graph_schema_matches_golden`,
  `emit_json_symbols_module_graph_schema_matches_golden`,
  `emit_json_symbols_generic_method_schema_matches_golden`,
  `emit_json_typed_generic_method_schema_matches_golden`,
  `emit_json_hir_generic_method_worklist_schema_matches_golden`,
  `emit_json_mir_generic_method_worklist_schema_matches_golden`,
  `emit_json_layout_generic_option_schema_matches_golden`,
  `emit_json_layout_generic_result_schema_matches_golden`, and
  `emit_json_layout_nested_generic_result_schema_matches_golden`.
- generic names pinned by golden fixtures: `Box.get<T>`, `Box.replace<T>`,
  `Box<T>.impl`, `Box.copy<T>`, `Option.copy<T>`, `inner<T>`, `id<T>`,
  `12.id<i32>()`, `id_i32`, `Box.get_inner<T>`, `Box.get_inner_i32`,
  `inner_i32`, `Option<T>`, `unwrap_or<T>`, `Result<T, E>`,
  `unwrap_or<T, E>`, `Result.unwrap_or<T, E>`, `self: Self`,
  `Json<StaticString>`, `Json<Point>`, and `Point.encode__Json_Point`.
- diagnostics JSON: `docs/DIAGNOSTICS.md` catalogs JSON-stable public diagnostic codes
  only after golden fixtures pin the code and shape; broader diagnostic-code coverage is still required. Important anchors include
  `context.kind = "feature_gate"`,
  `emit_json_diagnostics_removed_return_schema_matches_golden`,
  `emit_json_diagnostics_behavior_derive_gate_schema_matches_golden`,
  `emit_json_diagnostics_generic_association_gate_schema_matches_golden`,
  `emit_json_diagnostics_typed_allocator_effect_gate_schema_matches_golden`,
  `emit_json_diagnostics_dynamic_string_gate_schema_matches_golden`, and
  `emit_json_diagnostics_generic_function_arity_schema_matches_golden`.
- build/target JSON:
  `emit_json_build_graph_project_schema_matches_golden`,
  `emit_json_build_graph_host_effect_schema_matches_golden`,
  `emit_json_build_graph_target_metadata_schema_matches_golden`,
  `emit_json_target_yaml_validates_minimal_target_schema`,
  `emit_json_target_yaml_validates_backend_schema`,
  `emit_json_target_yaml_validates_c_backend_flags`,
  `emit_json_target_yaml_backend_schema_matches_golden`,
  `emit_json_target_yaml_rejects_empty_c_backend_flags`,
  `emit_json_target_yaml_rejects_layout_overrides`, and
  `emit_json_target_yaml_rejects_unsupported_backend_codegen`.

## Build Graph

`build.zen` is constrained. `zen check build.zen` validates a deterministic
graph and verifies declared target sources exist. `zen emit build.zen` emits C
for one target. `zen build build.zen` compiles executable targets, and direct
`zen build.zen` aliases that path. `zen test build.zen` compiles and runs test
targets. Executable target dependencies compile before their dependents;
dependency cycles are rejected before execution. test target execution, target C
emission, dependency-ordered multi-executable build tests, library target graph
emission, host-effect arrays, and legacy `emit-json ast|symbols|typed|diagnostics`
rejection are the current proof shape.

Constrained `build.zen` execution already has positive and negative evidence:
Deterministic build graph compiles executable and test targets, while build
scripts using undeclared host side effects are rejected.

## Feature Matrix

| Feature | Status | Gate |
|---|---|---|
| Lexer/parser for tested fixtures | implemented | Unit and integration tests |
| Local module loading | implemented | Resolver/module/privacy tests |
| Typechecked C backend for tested fixtures | implemented | `cargo test --tests` |
| README, contributor, docs shape | implemented | `tests/docs_truth.rs` |
| Strict resolver, symbol IDs, privacy | implemented | Resolver/module/privacy tests |
| AST/symbols/typed/diagnostics/HIR/MIR/layout JSON | constrained | Golden schemas and hand-authored JSON rejection tests |
| Target/build YAML validation | constrained | Target schema and backend validation tests |
| Build graph JSON emission | constrained | Deterministic graph schema tests |
| Developer UX and Agent UX | constrained/gated | Existing public docs and repo hygiene tests prevent unsupported `zen-lsp` claims, stale generated editor packages, and duplicate public examples |
| Behaviors and type association | gated | Positive/negative behavior solver tests |
| `Sync/Async effects` | gated | Effect checker positive/negative tests still required |
| `Typed allocators` | gated | Allocator ownership/lowering tests still required |
| Comptime type matching | gated | Type metadata and derive tests still required |
| Ownership and raw pointer operations | gated | Ownership/resource tests still required |
| Host syscalls | gated | Host-effect declaration and ABI tests still required |
| Actors in std | gated | Mailbox, scheduling, supervisor tests |
| `build.zen` check/emit/build/test/direct execution | constrained | Deterministic graph validation and execution tests |
| Existing broad stdlib files | experimental | Must compile before promotion |
| Formatter, package manager, alternate backends | removed | Reintroduce only with tests and binaries |

## Required Test Backlog

Every remaining v1 effect/type-match/allocator/actor/IR claim needs at least one
Planned Positive Test and one Planned Negative Test before implementation.

| Area | Planned Positive Test | Planned Negative Test |
|---|---|---|
| `Sync/Async effects` | Async function may enqueue, yield, and call async operation through checked APIs | Sync function calling async operation without blocking boundary is rejected |
| `Typed allocators` | `Allocator<i32, Sync>` returns a checked pointer result and propagates into a container | `Allocator<i32, Sync>` cannot satisfy an `Allocator<i32, Async>` parameter |
| Type matching | `to_json<T>` derive branches on struct and enum metadata | Ambiguous or unreachable type-match arm is diagnosed |
| Generated/fallback behavior association | Generated `Json<T>` derive fallback is used only when no explicit impl exists | Missing or ambiguous generated/fallback behavior impl is rejected |
| Actors in std | Actor mailbox send/receive works with scheduler and allocator integration | Actor using async mailbox from sync-only context is rejected |
| JSON/YAML IR boundaries | Checked layout JSON, checked MIR JSON, and target YAML validate against schemas | Hand-authored JSON IR cannot override compiler-owned types or layouts |

Generated/fallback behavior association syntax is reserved but not implemented:
`Type.derive(Json)` currently parses into a reserved AST declaration and then
reports an explicit resolver gate, covered by
`parser::tests::parse_generated_behavior_derive_association`,
`resolver_gates_generated_behavior_derive_association`,
`emit_json_diagnostics_spans_full_gated_behavior_derive_association`,
`emit_json_diagnostics_spans_full_gated_generic_association_target`, and
`emit_json_diagnostics_generic_association_gate_schema_matches_golden`.

## Stdlib Gate

Files under `stdlib/` are experimental unless a test proves they parse, typecheck,
and build through the same compiler path as user modules. Aspirational stdlib
APIs must not be described as implemented until promoted by tests.
