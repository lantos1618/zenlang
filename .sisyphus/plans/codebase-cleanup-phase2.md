# Codebase Cleanup Phase 2: AST Fields Trait + Remaining Dedup

## TL;DR

> **Quick Summary**: Move comptime field extraction logic onto AST types via an `AstFields` trait with lightweight `FieldValue` intermediate representation. This eliminates the 778-line `fields.rs` that manually hardcodes every AST variant, replacing it with ~30 lines of thin shim code. Also includes LSP handler boilerplate dedup and codegen normalization dedup.
>
> **Deliverables**:
> - New `src/ast/fields.rs` with `FieldValue` enum + `AstFields` trait
> - `AstFields` implementations on Expression, Statement, Declaration, AstType, Pattern
> - `FieldValue → ComptimeValue` converter in comptime/helpers.rs
> - `fields.rs` rewritten as 5 thin shim functions (~30 lines total)
> - LSP handler macro to eliminate 20+ handler boilerplate
> - Codegen integer width normalization consolidated
>
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: Task 1 → Task 2 → Task 3 → Task 4

---

## Context

### Original Request
Eliminate code duplication across the zenlang codebase. Phase 1 completed 8 refactors (literals.rs, hashmap.zen, math.zen, atomic.zen, primary.rs, LSP refs/highlight, intrinsics.rs, type_resolution.rs). Phase 2 addresses the remaining items: comptime fields.rs trait-based refactor, LSP handler boilerplate, and codegen normalization.

### Interview Summary
**Key Decisions**:
- User chose Option 1: Intermediate `FieldValue` representation in `src/ast/` — AST types own their field definitions without circular dependency on ComptimeValue
- `FieldValue` mirrors `ComptimeValue` primitives but adds `Expr/Stmt/Decl/Type/Pat` variants for AST nodes
- Conversion from `FieldValue → ComptimeValue` is a thin ~40 line function

### Research Findings
- `BehaviorMethod` and `TraitMethod` in `declarations.rs` are structurally identical (`{name, params: Vec<Parameter>, return_type}`) — can share a `ProtocolMethodLike` trait
- `MatchArm`, `ConditionalArm`, `PatternArm` are structurally identical — already have an `ArmLike` trait in helpers.rs that should move to `fields.rs`
- 20+ LSP handlers share identical boilerplate: parse params, lock store, get document, handle missing
- `normalize_int_widths` and `normalize_int_widths_for_logical` in codegen are 95% identical

---

## Work Objectives

### Core Objective
Make AST types self-describing for comptime introspection, eliminating the disconnected 778-line hardcoded mapping file.

### Concrete Deliverables
- `src/ast/fields.rs` — new file with `FieldValue` enum, `AstFields` trait, shared helpers
- `src/ast/expressions.rs` — `impl AstFields for Expression`
- `src/ast/statements.rs` — `impl AstFields for Statement`
- `src/ast/declarations.rs` — `impl AstFields for Declaration`
- `src/ast/types.rs` — `impl AstFields for AstType`
- `src/ast/patterns.rs` — `impl AstFields for Pattern`
- `src/ast/mod.rs` — register new module
- `src/comptime/meta/helpers.rs` — add `field_value_to_comptime` converter
- `src/comptime/meta/fields.rs` — rewrite as thin shims (~30 lines)

### Definition of Done
- [ ] `cargo check -p zen` passes
- [ ] `cargo test --lib` passes (143 tests, 0 failures)
- [ ] Comptime meta tests pass (`comptime::meta::tests::*` and `comptime::integration_tests::*`)
- [ ] `fields.rs` is under 50 lines (currently 778)

### Must Have
- Field names IDENTICAL to current `fields.rs` output (any mismatch breaks comptime programs)
- `ArmLike` trait moved from `helpers.rs` to `fields.rs` (or shared)
- `ProtocolMethodLike` trait for Behavior/Trait method dedup
- All existing helpers in `helpers.rs` (`field_info`, `ast_expr`, etc.) preserved

### Must NOT Have (Guardrails)
- No circular dependency: `src/ast/` must NOT import anything from `src/comptime/`
- No proc-macros — use manual trait implementations
- No changes to ComptimeValue enum itself
- No changes to how comptime interpreter calls `expression_fields` etc. (same API)
- No removal of `variant_name()` methods on AST types (used elsewhere)

---

## Verification Strategy

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: Tests-after (existing tests verify correctness)
- **Framework**: cargo test

### Agent-Executed QA Scenarios (MANDATORY)

```
Scenario: All comptime meta tests still pass
  Tool: Bash
  Steps:
    1. cargo test --lib -- comptime::meta::tests 2>&1
    2. Assert: all tests pass, 0 failures
    3. Specifically verify: test_fields_binary_op, test_fields_function_call,
       test_fields_function_declaration, test_fields_integer, test_fields_program,
       test_fields_variable_declaration
  Expected Result: All 6 field-related tests pass
  Evidence: Terminal output captured

Scenario: All comptime integration tests still pass
  Tool: Bash
  Steps:
    1. cargo test --lib -- comptime::integration_tests 2>&1
    2. Assert: all tests pass, 0 failures
  Expected Result: All integration tests pass
  Evidence: Terminal output captured

Scenario: Full test suite shows no regressions
  Tool: Bash
  Steps:
    1. cargo test --lib 2>&1
    2. Assert: 143 passed, 0 failed
  Expected Result: Same pass count as before refactor
  Evidence: Terminal output captured

Scenario: fields.rs is under 50 lines
  Tool: Bash
  Steps:
    1. wc -l src/comptime/meta/fields.rs
    2. Assert: line count < 50
  Expected Result: Dramatic reduction from 778 lines
  Evidence: wc output
```

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
├── Task 1: Create src/ast/fields.rs (FieldValue + AstFields + shared helpers)
└── Task 5: LSP handler boilerplate macro (independent)

Wave 2 (After Wave 1 Task 1):
├── Task 2: Implement AstFields for Expression
├── Task 3: Implement AstFields for Statement, Declaration, AstType, Pattern

Wave 3 (After Wave 2):
└── Task 4: Add converter + rewrite fields.rs as thin shims

Wave 4 (Independent):
└── Task 6: Codegen normalization dedup
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 1 | None | 2, 3 | 5, 6 |
| 2 | 1 | 4 | 3 |
| 3 | 1 | 4 | 2 |
| 4 | 2, 3 | None | 5, 6 |
| 5 | None | None | 1, 6 |
| 6 | None | None | 1, 5 |

---

## TODOs

- [ ] 1. Create `src/ast/fields.rs` with FieldValue enum, AstFields trait, and shared helpers

  **What to do**:
  - Create new file `src/ast/fields.rs`
  - Define `FieldValue` enum with variants: I8, I16, I32, I64, U8, U16, U32, U64, F32, F64, Bool, String, Array(Vec<FieldValue>), Struct { name: String, fields: HashMap<String, FieldValue> }, Expr(Box<Expression>), Stmt(Box<Statement>), Decl(Box<Declaration>), Type(Box<AstType>), Pat(Box<Pattern>), Null
  - Add convenience constructors on FieldValue: `expr()`, `boxed_expr()`, `stmt()`, `ty()`, `pat()`, `opt_expr()`, `opt_type()`, `opt_pattern()`, `opt_label()`, `string_array()`, `expr_array()`, `stmt_array()`, `type_array()`, `pat_array()`
  - Define `pub trait AstFields { fn ast_fields(&self) -> Vec<(&'static str, FieldValue)>; }`
  - Move `ArmLike` trait here (from `src/comptime/meta/helpers.rs`) with impls for MatchArm, ConditionalArm, PatternArm
  - Add `match_arms_fields<A: ArmLike>(struct_name, scrutinee, arms)` helper
  - Add `type_params_fields(tps)` helper (TypeParameter → FieldValue::Struct array)
  - Add `function_arg_field(name, ty)` helper
  - Add `parameter_field(p: &Parameter)` helper
  - Add `methods_field(methods: &[Function])` helper (wraps as Decl array)
  - Add `ProtocolMethodLike` trait with `method_name()`, `method_params()`, `method_return_type()`
  - Impl `ProtocolMethodLike` for `BehaviorMethod` and `TraitMethod`
  - Add `protocol_methods_field(struct_name, methods)` helper
  - Register in `src/ast/mod.rs`: add `pub mod fields;` and `pub use fields::{AstFields, FieldValue};`

  **Must NOT do**:
  - Do NOT import anything from `src/comptime/`
  - Do NOT use ComptimeValue anywhere in this file
  - Do NOT modify any existing AST type definitions

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 5, 6)
  - **Blocks**: Tasks 2, 3
  - **Blocked By**: None

  **References**:
  - `src/comptime/meta/helpers.rs` — current helper functions to replicate as FieldValue-based
  - `src/ast/declarations.rs:78-82` — BehaviorMethod struct definition
  - `src/ast/declarations.rs:93-97` — TraitMethod struct (identical to BehaviorMethod)
  - `src/ast/expressions.rs:245-264` — MatchArm/PatternArm/ConditionalArm structs
  - `src/ast/types.rs:115-124` — TypeParameter and TraitConstraint structs

  **Acceptance Criteria**:
  - [ ] File created at `src/ast/fields.rs`
  - [ ] `src/ast/mod.rs` updated with `pub mod fields;` and re-export
  - [ ] `cargo check -p zen` passes

  **Agent-Executed QA Scenarios**:
  ```
  Scenario: Module compiles cleanly
    Tool: Bash
    Steps:
      1. cargo check -p zen 2>&1
      2. Assert: exit code 0, no errors
    Expected Result: Clean compilation
    Evidence: Terminal output
  ```

  **Commit**: YES (groups with 2, 3, 4)
  - Message: `refactor(ast): add FieldValue enum + AstFields trait for self-describing AST nodes`
  - Files: `src/ast/fields.rs`, `src/ast/mod.rs`

- [ ] 2. Implement AstFields for Expression

  **What to do**:
  - In `src/ast/expressions.rs`, add `use super::fields::{AstFields, FieldValue, match_arms_fields};` and `use std::collections::HashMap;`
  - Implement `impl AstFields for Expression` with a match on all variants
  - CRITICAL: Read `src/comptime/meta/fields.rs` function `expression_fields` (lines 11-333) — field names must be IDENTICAL
  - Key patterns:
    - Integer8(v) → `[("value", FieldValue::I8(*v))]`, same for all literal types
    - Boolean(v) → `[("value", FieldValue::Bool(*v))]`
    - String(v) → `[("value", FieldValue::String(v.clone()))]`
    - Identifier(name) → `[("name", FieldValue::String(name.clone()))]`
    - Unit|None|StdReference|BuiltinReference|ThisReference → `[]`
    - BinaryOp → `[("left", boxed_expr), ("op", String(op.to_string())), ("right", boxed_expr)]`
    - FunctionCall → `[("name", str), ("type_args", type_array), ("args", expr_array)]`
    - MethodCall → `[("object", boxed_expr), ("method", str), ("type_args", type_array), ("args", expr_array)]`
    - QuestionMatch → `match_arms_fields("MatchArm", scrutinee, arms)`
    - Conditional → `match_arms_fields("ConditionalArm", scrutinee, arms)`
    - PatternMatch → `match_arms_fields("PatternArm", scrutinee, arms)`
    - AddressOf|Dereference|PointerDereference|PointerAddress|CreateReference|CreateMutableReference|StringLength|Comptime → `[("expr", boxed_expr)]`
    - Some(inner) → `[("inner", boxed_expr)]`
    - Return|Raise|Defer → `[("expr", boxed_expr)]`
    - StructLiteral → name + fields as Array of Struct{name: "StructFieldInit", fields: {name, value}}
    - StructField → `[("struct_expr", boxed_expr), ("field", str)]`
    - ArrayLiteral → `[("elements", expr_array)]`
    - ArrayIndex → `[("array", boxed_expr), ("index", boxed_expr)]`
    - ArrayConstructor → `[("element_type", ty)]`
    - VecConstructor → element_type + size(as i64) + initial_values (Array or empty Array)
    - DynVecConstructor → element_types(type_array) + allocator(boxed_expr) + initial_capacity(opt boxed_expr)
    - EnumVariant → enum_name + variant + payload(opt_expr)
    - EnumLiteral → variant + payload(opt_expr)
    - MemberAccess → object(boxed_expr) + member(str)
    - StringInterpolation → parts as Array of Struct{name: "StringPart", fields: {kind: "Literal"/"Interpolation", value/expr}}
    - Range → start(boxed_expr) + end(boxed_expr) + inclusive(bool)
    - Loop → body(boxed_expr)
    - CollectionLoop → collection(boxed_expr) + param_name(str) + param_type(opt_type) + index_name(str or empty) + index_type(type or Null) + body(boxed_expr)
    - Closure → params as Array of Struct{name: "ClosureParam", fields: {name, param_type}} + return_type(opt_type) + body(boxed_expr)
    - Block → statements(stmt_array)
    - Break → label(opt_label) + value(opt_expr)
    - Continue → label(opt_label)

  **Must NOT do**:
  - Do NOT change field names from what `expression_fields` currently returns
  - Do NOT modify the Expression enum itself
  - Do NOT remove variant_name() method

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 3)
  - **Blocks**: Task 4
  - **Blocked By**: Task 1

  **References**:
  - `src/comptime/meta/fields.rs:11-333` — EXACT field names to match for Expression
  - `src/ast/expressions.rs:58-237` — Expression enum definition
  - `src/comptime/meta/helpers.rs:110-139` — match_arms_to_fields (now match_arms_fields in fields.rs)

  **Acceptance Criteria**:
  - [ ] `impl AstFields for Expression` covers ALL variants
  - [ ] `cargo check -p zen` passes
  - [ ] Field names match `expression_fields` exactly

  **Agent-Executed QA Scenarios**:
  ```
  Scenario: Compilation passes with Expression impl
    Tool: Bash
    Steps:
      1. cargo check -p zen 2>&1
      2. Assert: exit code 0
    Expected Result: Clean compile
    Evidence: Terminal output
  ```

  **Commit**: NO (groups with Task 4)

- [ ] 3. Implement AstFields for Statement, Declaration, AstType, Pattern

  **What to do**:
  - In `src/ast/statements.rs`: add imports, impl AstFields for Statement
    - Match `src/comptime/meta/fields.rs:336-437` (statement_fields) EXACTLY
    - Expression/Return → `[("expr", expr)]`
    - VariableDeclaration → name, var_type(opt), initializer(opt), is_mutable, declaration_type(to_string)
    - VariableAssignment → name, value
    - PointerAssignment → pointer, value
    - Loop → kind (Infinite string or Struct with condition), label(opt), body(stmt_array)
    - Break/Continue → label(opt)
    - ComptimeBlock/Block → statements(stmt_array)
    - ModuleImport → alias, module_path
    - Defer → statement(boxed_stmt)
    - ThisDefer → expr
    - DestructuringImport → names(string_array), source(expr)

  - In `src/ast/declarations.rs`: add imports, impl AstFields for Declaration
    - Match `src/comptime/meta/fields.rs:439-605` (declaration_fields) EXACTLY
    - Function(f) → use function fields inline: name, type_params, args (FunctionArg array), return_type, body(stmt_array), is_varargs, is_public
    - ExternalFunction → name, args(type_array), return_type, is_varargs
    - Struct → name, type_params, fields (StructField array with name, field_type, is_mutable, default_value), methods
    - Enum → name, type_params, variants (EnumVariant array), methods, required_traits(string_array)
    - Behavior → name, type_params, methods via protocol_methods_field("BehaviorMethod", &b.methods)
    - Trait → name, type_params, methods via protocol_methods_field("TraitMethod", &t.methods)
    - TraitImplementation → type_name, trait_name, type_params, methods
    - TraitRequirement → type_name, trait_name
    - ImplBlock → type_name, type_params, methods
    - ComptimeBlock → statements(stmt_array)
    - Constant → name, value(expr), const_type(opt_type)
    - ModuleImport → alias, module_path
    - Export → symbols(string_array)
    - TypeAlias → name, type_params, target_type

  - In `src/ast/types.rs`: add imports, impl AstFields for AstType
    - Match `src/comptime/meta/fields.rs:607-708` (type_fields) EXACTLY
    - All primitives (I8..StdModule) → `[]`
    - Slice → element_type(boxed_ty)
    - FixedArray → element_type(boxed_ty), size(i64)
    - Function → args(type_array), return_type(boxed_ty)
    - FunctionPointer → param_types(type_array), return_type(boxed_ty)
    - Struct → name, fields (StructTypeField array with name, field_type)
    - Enum → name, variants (EnumVariant array with name, payload)
    - Ref → inner(boxed_ty)
    - Range → start_type, end_type, inclusive
    - Generic → name, type_args(type_array)
    - EnumType → name

  - In `src/ast/patterns.rs`: add imports, impl AstFields for Pattern
    - Match `src/comptime/meta/fields.rs:710-777` (pattern_fields) EXACTLY
    - Literal → value(expr)
    - Identifier → name(str)
    - Struct → name, fields (PatternField array with name, pattern)
    - EnumVariant → enum_name, variant, payload(opt_pattern)
    - Wildcard → `[]`
    - EnumLiteral → variant, payload(opt_pattern)
    - Or/Tuple → patterns(pat_array)
    - Range → start(boxed_expr), end(boxed_expr), inclusive
    - Binding → name, pattern(boxed_pat)
    - Type → type_name, binding(opt_label)
    - Guard → pattern(boxed_pat), condition(boxed_expr)

  **Must NOT do**:
  - Do NOT modify any enum definitions or existing methods
  - Do NOT change field names from what current fields.rs returns

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 2)
  - **Blocks**: Task 4
  - **Blocked By**: Task 1

  **References**:
  - `src/comptime/meta/fields.rs:336-437` — statement_fields exact field names
  - `src/comptime/meta/fields.rs:439-605` — declaration_fields exact field names
  - `src/comptime/meta/fields.rs:607-708` — type_fields exact field names
  - `src/comptime/meta/fields.rs:710-777` — pattern_fields exact field names
  - `src/comptime/meta/helpers.rs:151-172` — function_to_fields structure
  - `src/comptime/meta/helpers.rs:174-200` — type_params_to_array structure
  - `src/comptime/meta/helpers.rs:202-209` — methods_to_array structure

  **Acceptance Criteria**:
  - [ ] All 4 impl blocks compile
  - [ ] `cargo check -p zen` passes

  **Commit**: NO (groups with Task 4)

- [ ] 4. Add FieldValue→ComptimeValue converter, rewrite fields.rs as shims

  **What to do**:
  - In `src/comptime/meta/helpers.rs`:
    - Add `use crate::ast::FieldValue;`
    - Add `pub fn field_value_to_comptime(fv: FieldValue) -> ComptimeValue` function:
      ```
      FieldValue::I8(v) => ComptimeValue::I8(v),
      FieldValue::I16(v) => ComptimeValue::I16(v),
      ... (all primitives 1:1) ...
      FieldValue::String(v) => ComptimeValue::String(v),
      FieldValue::Bool(v) => ComptimeValue::Bool(v),
      FieldValue::Array(arr) => ComptimeValue::Array(arr.into_iter().map(field_value_to_comptime).collect()),
      FieldValue::Struct { name, fields } => ComptimeValue::Struct {
          name,
          fields: fields.into_iter().map(|(k, v)| (k, field_value_to_comptime(v))).collect(),
      },
      FieldValue::Expr(e) => ast_expr(*e),
      FieldValue::Stmt(s) => ast_stmt(*s),
      FieldValue::Decl(d) => ast_node(ASTNodeValue::Declaration(*d)),
      FieldValue::Type(t) => ast_type(*t),
      FieldValue::Pat(p) => ast_pattern(*p),
      FieldValue::Null => ComptimeValue::Null,
      ```
    - Check if `ArmLike`, `function_to_fields`, `match_arms_to_fields`, `type_params_to_array`, `methods_to_array`, `parameter_to_value` are used outside of fields.rs (grep for each). Remove only if unreferenced.

  - Rewrite `src/comptime/meta/fields.rs` entirely:
    ```rust
    use crate::ast::{AstFields, AstType, Declaration, Expression, Pattern, Statement};
    use crate::error::Result;
    use super::helpers::{field_info, field_value_to_comptime};
    use crate::comptime::values::ComptimeValue;

    fn fields_to_comptime(fields: Vec<(&str, crate::ast::FieldValue)>) -> Vec<ComptimeValue> {
        fields.into_iter()
            .map(|(name, val)| field_info(name, field_value_to_comptime(val)))
            .collect()
    }

    pub fn expression_fields(expr: &Expression) -> Result<Vec<ComptimeValue>> {
        Ok(fields_to_comptime(expr.ast_fields()))
    }
    pub fn statement_fields(stmt: &Statement) -> Result<Vec<ComptimeValue>> {
        Ok(fields_to_comptime(stmt.ast_fields()))
    }
    pub fn declaration_fields(decl: &Declaration) -> Result<Vec<ComptimeValue>> {
        Ok(fields_to_comptime(decl.ast_fields()))
    }
    pub fn type_fields(ty: &AstType) -> Result<Vec<ComptimeValue>> {
        Ok(fields_to_comptime(ty.ast_fields()))
    }
    pub fn pattern_fields(pat: &Pattern) -> Result<Vec<ComptimeValue>> {
        Ok(fields_to_comptime(pat.ast_fields()))
    }
    ```

  **Must NOT do**:
  - Do NOT change the function signatures of the 5 public functions
  - Do NOT remove helper functions that are still used elsewhere

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (sequential after 2, 3)
  - **Blocks**: None
  - **Blocked By**: Tasks 2, 3

  **References**:
  - `src/comptime/meta/helpers.rs` — add converter here, check what to clean up
  - `src/comptime/meta/fields.rs` — rewrite target
  - `src/comptime/values.rs:96-106` — ASTNodeValue enum for Decl conversion

  **Acceptance Criteria**:
  - [ ] `cargo check -p zen` passes
  - [ ] `cargo test --lib` → 143 passed, 0 failed
  - [ ] `src/comptime/meta/fields.rs` is under 50 lines
  - [ ] All comptime::meta::tests pass
  - [ ] All comptime::integration_tests pass

  **Agent-Executed QA Scenarios**:
  ```
  Scenario: Full test suite passes
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1
      2. Assert: "143 passed; 0 failed"
    Expected Result: Zero regressions
    Evidence: Terminal output

  Scenario: fields.rs is dramatically smaller
    Tool: Bash
    Steps:
      1. wc -l src/comptime/meta/fields.rs
      2. Assert: < 50 lines (was 778)
    Expected Result: ~95% reduction
    Evidence: wc output

  Scenario: Comptime meta tests verify field correctness
    Tool: Bash
    Steps:
      1. cargo test --lib -- comptime::meta::tests 2>&1
      2. Assert: test_fields_binary_op, test_fields_function_call,
         test_fields_function_declaration, test_fields_integer,
         test_fields_program, test_fields_variable_declaration all pass
    Expected Result: All field extraction tests pass with identical output
    Evidence: Terminal output
  ```

  **Commit**: YES
  - Message: `refactor(comptime): move field extraction onto AST types via AstFields trait`
  - Files: `src/ast/fields.rs`, `src/ast/mod.rs`, `src/ast/expressions.rs`, `src/ast/statements.rs`, `src/ast/declarations.rs`, `src/ast/types.rs`, `src/ast/patterns.rs`, `src/comptime/meta/helpers.rs`, `src/comptime/meta/fields.rs`
  - Pre-commit: `cargo test --lib`

- [ ] 5. LSP handler boilerplate macro

  **What to do**:
  - Read 3-4 LSP handlers to extract the exact boilerplate pattern
  - Create a macro `lsp_handler!` (or helper function) in `src/lsp/helpers.rs` that handles:
    1. Parse params via `try_parse_params`
    2. Lock store via `try_lock`
    3. Look up document by URI
    4. Call a closure with `(&doc, &params, &req)` for the actual logic
    5. Return `null_response` on missing doc
  - Refactor the simplest handlers first: `hover`, `signature_help`, `inlay_hints`
  - Then refactor: `definition`, `highlight`, `references`
  - Count handlers converted and lines saved

  **Must NOT do**:
  - Do NOT change any handler behavior
  - Do NOT break LSP protocol compliance

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: None
  - **Blocked By**: None

  **References**:
  - `src/lsp/helpers.rs` — existing helper functions
  - `src/lsp/hover/mod.rs:38-62` — typical handler boilerplate pattern
  - `src/lsp/signature_help.rs:20-43` — identical boilerplate
  - `src/lsp/inlay_hints.rs:14-28` — identical boilerplate
  - `src/lsp/navigation/definition.rs:37-70` — identical boilerplate
  - `src/lsp/navigation/highlight.rs:31-57` — already refactored, good reference

  **Acceptance Criteria**:
  - [ ] Macro/helper created in helpers.rs
  - [ ] At least 6 handlers converted
  - [ ] `cargo check -p zen` passes
  - [ ] `cargo test --lib` passes

  **Commit**: YES
  - Message: `refactor(lsp): extract handler boilerplate into lsp_handler helper`
  - Files: `src/lsp/helpers.rs`, `src/lsp/hover/mod.rs`, `src/lsp/signature_help.rs`, `src/lsp/inlay_hints.rs`, + others
  - Pre-commit: `cargo test --lib`

- [ ] 6. Codegen integer width normalization dedup

  **What to do**:
  - Read `src/codegen/llvm/binary_ops.rs` functions `normalize_int_widths` (lines 73-96) and `normalize_int_widths_for_logical` (lines 99-130)
  - They are 95% identical — only difference is the extension type (sign-extend vs zero-extend)
  - Create a single `normalize_int_widths_impl(builder, lhs, rhs, use_sign_extend: bool)` function
  - `normalize_int_widths` calls it with `use_sign_extend: true`
  - `normalize_int_widths_for_logical` calls it with `use_sign_extend: false`

  **Must NOT do**:
  - Do NOT change behavior of either function
  - Do NOT change their public signatures

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 5)
  - **Blocks**: None
  - **Blocked By**: None

  **References**:
  - `src/codegen/llvm/binary_ops.rs:73-130` — both functions to consolidate

  **Acceptance Criteria**:
  - [ ] Single impl function created
  - [ ] Both public functions delegate to it
  - [ ] `cargo check -p zen` passes

  **Commit**: YES
  - Message: `refactor(codegen): consolidate normalize_int_widths and normalize_int_widths_for_logical`
  - Files: `src/codegen/llvm/binary_ops.rs`

---

## Commit Strategy

| After Task | Message | Verification |
|------------|---------|--------------|
| 1 (if standalone) | `refactor(ast): add FieldValue enum + AstFields trait` | cargo check |
| 4 (groups 1-4) | `refactor(comptime): move field extraction onto AST types via AstFields trait` | cargo test --lib (143 pass) |
| 5 | `refactor(lsp): extract handler boilerplate into lsp_handler helper` | cargo test --lib |
| 6 | `refactor(codegen): consolidate normalize_int_widths` | cargo check |

---

## Success Criteria

### Verification Commands
```bash
cargo check -p zen          # Expected: clean compile
cargo test --lib             # Expected: 143 passed, 0 failed
wc -l src/comptime/meta/fields.rs  # Expected: < 50 lines
```

### Final Checklist
- [ ] AST types are self-describing via AstFields trait
- [ ] fields.rs is a thin shim, not a 778-line hardcoded mapping
- [ ] No circular dependencies (ast/ does not import comptime/)
- [ ] All field names identical to before (comptime programs unaffected)
- [ ] LSP handler boilerplate reduced
- [ ] Codegen normalization consolidated
- [ ] All 143 tests pass
