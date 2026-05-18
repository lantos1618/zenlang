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
| `E5000` | generic inference conflict, including conflicting inferred type arguments for generic methods. | `tests/fixtures/ir_json/diagnostics_generic_result_method_inference.golden.json` |
| `E5001` | generic type-argument arity mismatch for functions, methods, structs, enums, annotations, and constructors. | `tests/fixtures/ir_json/diagnostics_generic_result_method_arity.golden.json` |
| `E6004` | generic behavior-bound failure when a concrete type does not satisfy a required behavior bound. | `tests/fixtures/ir_json/diagnostics_generic_result_method_bound.golden.json` |

## Catalog Rules

- Add a focused golden fixture before listing a code as JSON-stable.
- Keep suggested-fix `kind` values stable once tools can apply them.
- Keep `context.kind` values enum-like and lower snake case.
- Do not list internal resolver/typechecker hardening codes here until a public
  diagnostics JSON fixture pins the code and shape.
- Prefer adding narrowly scoped catalog entries over claiming the full compiler
  diagnostic space is stable.
