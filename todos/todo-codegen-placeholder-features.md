# TODO: Implement Codegen Placeholders for Core Language Features

These are the remaining placeholder methods in `src/codegen/llvm/expressions.rs` that need real implementations for full language support:

- [x] `compile_array_literal` — Heap-allocate and store array elements (done)
- [x] `compile_array_index` — Index into arrays and load values (done)
- [ ] `compile_member_access` — Access struct fields
- [ ] `compile_enum_variant` — Construct enum variants
- [ ] `compile_pattern_match` — Pattern matching codegen
- [ ] `compile_range_expression` — Range value codegen
- [ ] `compile_comptime_expression` — Compile-time expression evaluation

**Order of implementation:**
1. `compile_member_access`
2. `compile_enum_variant`
3. `compile_pattern_match`
4. `compile_range_expression`
5. `compile_comptime_expression`

---

Work through these in order for maximum language feature coverage. 