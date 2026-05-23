# Zen Diagnostics Catalog

This catalog documents diagnostic codes that are stable at the public JSON
boundary. A code is JSON-stable only after a golden fixture pins its `code`,
message shape, span shape, notes, context, and suggested fixes.

Diagnostic text may become more precise, but tools and agents should treat the
code, severity, context kind, and suggested-fix kind as the compatibility
surface once listed here.

Fixture evidence lives under `tests/fixtures/ir_json/diagnostics_*.golden.json`;
the table lists representative anchors, not an exhaustive fixture inventory.

## JSON-Stable Codes

| Code | Meaning | Representative JSON Evidence |
|---|---|---|
| `E2000` | Syntax-level removed syntax or reserved syntax. Removed source keywords include `replace_removed_return_with_final_expression`; gated behavior association paths include `feature_gate` context. | `diagnostics_return`, `diagnostics_behavior_derive_gate`, `diagnostics_generic_association_gate` |
| `E0203` | gated compiler-owned intrinsic call: comptime type matching, reserved async scheduler intrinsics, atomic intrinsics, raw syscalls, raw allocation intrinsics, byte-memory intrinsics, and raw pointer intrinsics. | `diagnostics_type_match_gate`, `diagnostics_async_intrinsic_gate`, `diagnostics_atomic_gate`, `diagnostics_syscall_gate`, `diagnostics_raw_allocate_gate`, `diagnostics_byte_memory_gate`, `diagnostics_raw_pointer_gate` |
| `E3053` | gated range expression. | `tests/fixtures/ir_json/diagnostics_range_gate.golden.json` |
| `E3054` | gated Result propagation through `.raise()`. | `tests/fixtures/ir_json/diagnostics_raise_gate.golden.json` |
| `E3055` | gated task waiting through `.await()`. | `tests/fixtures/ir_json/diagnostics_await_gate.golden.json` |
| `E4006` | non-exhaustive bool match diagnostics with `add_missing_bool_match_arm` suggested fixes. | `tests/fixtures/ir_json/diagnostics_missing_bool_match_arm.golden.json` |
| `E3500` | resolver validation failure for duplicate type associations and gated reserved type surfaces: Sync/Async effect modes, allocator-backed dynamic `String`, std actor framework types/imports, std allocator imports, and std Sync/Async runtime imports. | duplicate generic association fixtures, typed allocator/effect gates, dynamic string gates, actor/import/runtime gates |
| `E5000` | generic inference failure or conflict, including missing and conflicting inferred type arguments for generic functions and methods. | `tests/fixtures/ir_json/diagnostics_generic_function_inference.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_result_method_inference.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_function_inference_failure.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_method_inference_failure.golden.json` |
| `E5001` | generic type-argument arity mismatch for functions, methods, structs, enums, annotations, constructors, and behavior references. | generic function/method annotation fixtures, nested generic fixtures, struct/enum constructor fixtures, behavior requires/impl/extends fixtures |
| `E5002` | type arguments were supplied to a non-generic function, method, struct, enum, or behavior. | nongeneric annotation, constructor, function, builtin, module function, method, and `tests/fixtures/ir_json/diagnostics_nongeneric_requires_type_args.golden.json` |
| `E6004` | generic behavior-bound failure when a concrete type does not satisfy a required behavior bound. | `tests/fixtures/ir_json/diagnostics_generic_function_bound.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_result_method_bound.golden.json`, `tests/fixtures/ir_json/diagnostics_generic_enum_constructor_bound.golden.json` |
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
