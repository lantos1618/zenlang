# Zen Compiler Improvements — February 2026

**Date:** 2026-02-07
**Focus:** `comptime/mod.rs` architecture + compiler-wide cleanup
**Codebase:** ~41,300 LOC across 181 .rs files

---

## Executive Summary

The comptime module is the meta-programming heart of Zen — it enables compile-time code generation, AST introspection, and Zig-style comptime evaluation. At 2835 lines, it works but has accumulated structural debt that makes it harder to extend and reason about. This document identifies concrete improvements and implements them.

---

## 1. Critical Design Smells (comptime)

### 1.1 Break/Continue via Error Strings (SEVERITY: HIGH)

**Problem:** Break and continue use error-string abuse for control flow.

```rust
// Line 540-547: Break/Continue implemented as fake errors
Statement::Break { .. } => {
    Err(CompileError::ComptimeError("__break__".to_string(), None))
}
Statement::Continue { .. } => {
    Err(CompileError::ComptimeError("__continue__".to_string(), None))
}

// Line 1010-1018: Caught by string comparison
if msg == "__break__" { should_break = true; break; }
if msg == "__continue__" { break; }
```

**Why it's bad:**
- Conflates errors with control flow — a typo in `"__break__"` silently breaks loops
- `?` operator propagates these "errors" through call stacks unexpectedly
- Makes it impossible to distinguish real ComptimeErrors from control signals
- Prevents adding break-with-value (`break expr`) cleanly

**Fix:** Introduce a proper `ComptimeControlFlow` enum:
```rust
enum ComptimeControlFlow {
    Break(Option<ComptimeValue>),
    Continue,
    Return(ComptimeValue),
}
```

### 1.2 Environment Swapping Boilerplate (SEVERITY: MEDIUM)

**Problem:** The pattern `std::mem::replace(&mut self.env, child_env)` appears 6+ times.

```rust
// Pattern repeated at lines 634, 938, 1047, and more:
let saved_env = std::mem::replace(&mut self.env, child_env);
// ... do work ...
self.env = saved_env;  // manual restore, easy to forget on error paths
```

**Why it's bad:**
- If any code path returns early (via `?`), the environment is never restored
- Duplicated boilerplate obscures the actual logic
- Error-prone: one missed restore = scope corruption

**Fix:** Extract into a `with_scope` method that uses RAII or a closure:
```rust
fn with_scope<F, R>(&mut self, f: F) -> R
where F: FnOnce(&mut Self) -> R {
    let child_env = Environment::with_parent(self.env.clone());
    let saved = std::mem::replace(&mut self.env, child_env);
    let result = f(self);
    self.env = saved;
    result
}
```

### 1.3 AST Node Method Dispatch — Massive Duplication (SEVERITY: HIGH)

**Problem:** Lines 1456-1628 contain 7 nearly identical methods: `functions()`, `structs()`, `enums()`, `constants()`, `find_function()`, `find_struct()`, `find_enum()`. Each follows the same pattern:

1. Check if node is `Program`
2. Filter declarations by variant
3. Map to `ASTNode`

~170 lines that could be ~30.

**Fix:** Extract `filter_declarations()` and `find_declaration()` helpers.

### 1.4 Missing `Display` for `ComptimeValue` (SEVERITY: LOW)

**Problem:** String interpolation (line 706-713) has a hardcoded format dispatch instead of a `Display` impl.

### 1.5 Duplicate Array Index Handling (SEVERITY: LOW)

**Problem:** Lines 664-694 handle `I32` and `I64` identically, doubling the code.

**Fix:** Extract index to `usize` first, then handle once.

---

## 2. Compiler-Wide Issues

### 2.1 `#[allow(dead_code)]` Epidemic

130 `#[allow(dead_code)]` markers across the codebase. Key offenders:
- `Compiler` struct and all its methods — used via integration tests but not `main.rs`
- `CompileError` — 10 variants marked dead
- `ComptimeInterpreter` — struct itself marked dead

Most of these exist because `main.rs` duplicates the compilation pipeline instead of using `Compiler::run_pipeline()`. The `main.rs` at 1870 lines rebuilds the pipeline inline.

### 2.2 Remaining Technical Debt (from TECHNICAL_DEBT_AUDIT.md)

| Item | Status | Priority |
|------|--------|----------|
| stdlib_types.rs only parses 15/60+ files | Open | HIGH |
| String-based type checks in LSP | Partial | HIGH |
| Generic type substitution in stdlib resolution | Open | HIGH |
| Windows path compatibility | Open | LOW |
| TypeAliasRegistry missing | Open | MEDIUM |

---

## 3. Implementation Plan (This Session)

### Phase A: Comptime Control Flow (lines touched: ~100)
- [x] Add `ComptimeControlFlow` enum
- [x] Replace `"__break__"`/`"__continue__"` with proper variants
- [x] Update `execute_loop()` to match on enum
- [x] Update block expression to propagate properly

### Phase B: Environment Scoping (lines touched: ~60)
- [x] Add `with_scope()` method to `ComptimeInterpreter`
- [x] Replace all `mem::replace` + manual restore patterns

### Phase C: AST Node Method DRY (lines touched: ~150)
- [x] Extract `filter_program_declarations()` helper
- [x] Extract `find_program_declaration()` helper
- [x] Rewrite 7 methods using helpers

### Phase D: Small Cleanups (lines touched: ~40)
- [x] Implement `Display` for `ComptimeValue`
- [x] Unify array index handling
- [x] Remove unnecessary `#[allow(dead_code)]` where possible

---

## 4. Additional Work Completed (Session 2)

### Phase E: Comptime Cleanup
- [x] Removed dead `generate_code()` method (never called)
- [x] Removed self-hosting fantasy stubs (`@std.lexer`, `@std.parser`, `@std.ast`, `@std.type_checker`, `@std.codegen`)
- [x] Updated `@std` stubs to match actual stdlib directory structure (`collections`, `memory`, `sys`, `concurrency`)
- [x] Added `emit()` builtin + `push_declaration()` to wire up comptime code generation
- [x] `get_generated_declarations()` now uses `std::mem::take` (move, not clone)

### Phase F: stdlib_types.rs Directory Scanning (CRITICAL)
- [x] Replaced hardcoded 15-file list with recursive `scan_zen_files()` scanner
- [x] Now automatically discovers all 60+ `.zen` files in stdlib/

### Phase G: DRY Utilities
- [x] Created `src/name_utils.rs` with `split_module_path`, `split_method_path`, `base_name`, `leaf_name`, `strip_generics`
- [x] Added `is_test_name()` and `is_test_file()` for consistent test detection
- [x] Wired `code_lens.rs` and `document_store/mod.rs` to use centralized helpers

### Phase H: meta.rs Breakdown (1691 → 4 files)
- [x] Extracted `meta/helpers.rs` (shared builders)
- [x] Extracted `meta/fields.rs` (AST field extraction)
- [x] Extracted `meta/variants.rs` (variant name constants)
- [x] Added `ArmLike` trait to DRY 3 identical match arm blocks
- [x] Fixed `function_to_fields` to use `type_params_to_array` (was duplicated)
- [x] Inlined 5 thin variant_name wrapper functions
- [x] Added `opt_label()` helper (eliminated 6 identical blocks)

## 5. Still Not Addressed (Future Work)

- `main.rs` refactor (1870 lines → clap subcommands + modules)
- Generic type substitution in method resolution (Phase 3-4 of tech debt)
- Windows path compatibility
- String-based type checks in LSP (Phase 2 of tech debt — ~15 locations)
- TypeAliasRegistry for StaticString/String normalization
- Self-hosting preparation

---

*Document created 2026-02-07, updated 2026-02-07*
