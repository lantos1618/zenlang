# Zen Diagnostics Catalog

This catalog documents diagnostic codes that are stable at the public JSON
boundary. A code is JSON-stable only after a golden fixture pins its `code`,
message shape, span shape, notes, context, and suggested fixes.

Diagnostic text may become more precise, but tools and agents should treat the
code, severity, context kind, and suggested-fix kind as the compatibility
surface once listed here.

## JSON-Stable Codes

| Code | Meaning | Stable JSON Evidence |
|---|---|---|
| `E2000` | Syntax-level removed syntax or reserved syntax. The removed source keyword path includes the `replace_removed_return_with_final_expression` suggested fix, while gated behavior association paths include `feature_gate` context. | `tests/fixtures/ir_json/diagnostics_return.golden.json`, `tests/fixtures/ir_json/diagnostics_behavior_derive_gate.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_association_gate.golden.json` |
| `E0203` | gated compiler-owned intrinsic call, including comptime type matching, reserved async scheduler intrinsics, atomic intrinsics, raw syscalls, raw allocation intrinsics, byte-memory intrinsics, and raw pointer intrinsics before typed metadata/derive lowering, Sync/Async task lowering, memory-order rules, host-effect declarations, syscall ABI semantics, allocator ownership, pointer provenance, and layout semantics are implemented. | `tests/fixtures/ir_json/diagnostics_type_match_gate.golden.json`, `tests/fixtures/ir_json/diagnostics_async_intrinsic_gate.golden.json`, `tests/fixtures/ir_json/diagnostics_atomic_gate.golden.json`, `tests/fixtures/ir_json/diagnostics_syscall_gate.golden.json`, `tests/fixtures/ir_json/diagnostics_raw_allocate_gate.golden.json`, `tests/fixtures/ir_json/diagnostics_byte_memory_gate.golden.json`, `tests/fixtures/ir_json/diagnostics_raw_pointer_gate.golden.json` |
| `E3053` | gated range expression; range typing remains unavailable until range types and lowering semantics are implemented. | `tests/fixtures/ir_json/diagnostics_range_gate.golden.json` |
| `E3054` | gated Result propagation through `.raise()` until propagation typing and lowering semantics are implemented. | `tests/fixtures/ir_json/diagnostics_raise_gate.golden.json` |
| `E3055` | gated task waiting through `.await()` until Sync/Async effect checking and task lowering semantics are implemented. | `tests/fixtures/ir_json/diagnostics_await_gate.golden.json` |
| `E3500` | resolver validation failure, including duplicate explicit type association requirements/implementations and gated reserved type surfaces such as allocator-backed dynamic `String`. | `tests/fixtures/ir_json/diagnostics_duplicate_generic_requires.golden.json`, `tests/fixtures/ir_json/diagnostics_duplicate_generic_impl.golden.json`, `tests/fixtures/ir_json/diagnostics_typed_allocator_effect_gate.golden.json`, `tests/fixtures/ir_json/diagnostics_dynamic_string_gate.golden.json` |
| `E5000` | generic inference conflict, including conflicting inferred type arguments for generic methods. | `tests/fixtures/ir_json/diagnostics_generic_result_method_inference.golden.json` |
| `E5001` | generic type-argument arity mismatch for functions, methods, structs, enums, annotations, constructors, and behavior references. | `tests/fixtures/ir_json/diagnostics_generic_function_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_function_type_arg_annotation_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_method_type_arg_annotation_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_method_type_arg_annotation_missing_args.golden.json`, `tests/fixtures/ir_json/diagnostics_closure_param_annotation_type_arg_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_closure_return_annotation_missing_args.golden.json`, `tests/fixtures/ir_json/diagnostics_cast_target_annotation_type_arg_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_cast_target_annotation_missing_args.golden.json`, `tests/fixtures/ir_json/diagnostics_nested_generic_annotation_inner_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_nested_generic_instantiation_inner_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_function_type_parameter_annotation_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_function_type_return_annotation_missing_args.golden.json`, `tests/fixtures/ir_json/diagnostics_pointer_inner_generic_annotation_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_slice_inner_generic_annotation_missing_args.golden.json`, `tests/fixtures/ir_json/diagnostics_array_inner_generic_annotation_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_struct_local_annotation_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_struct_local_annotation_missing_args.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_enum_local_annotation_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_enum_local_annotation_missing_args.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_struct_constructor_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_struct_constructor_missing_args.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_enum_constructor_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_enum_constructor_missing_args.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_struct_annotation_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_enum_annotation_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_struct_annotation_missing_args.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_enum_annotation_missing_args.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_result_method_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_requires_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_impl_arity.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_extends_arity.golden.json` |
| `E6004` | generic behavior-bound failure when a concrete type does not satisfy a required behavior bound. | `tests/fixtures/ir_json/diagnostics_generic_result_method_bound.golden.json` |
| `E6007` | explicit type association `.requires` failure when a type does not implement the required concrete behavior. | `tests/fixtures/ir_json/diagnostics_generic_requires_missing_impl.golden.json` |
| `E6010` | behavior implementation coherence failure, including overlapping generic parent and child behavior implementations. | `tests/fixtures/ir_json/diagnostics_generic_behavior_overlap.golden.json` |

## Catalog Rules

- Add a focused golden fixture before listing a code as JSON-stable.
- Keep suggested-fix `kind` values stable once tools can apply them.
- Keep `context.kind` values enum-like and lower snake case.
- Do not list internal resolver/typechecker hardening codes here until a public
  diagnostics JSON fixture pins the code and shape.
- Prefer adding narrowly scoped catalog entries over claiming the full compiler
  diagnostic space is stable.
