# Sema Integration Refactor

**Goal:** Integrate typechecker into main compilation pipeline
**Status:** IN PROGRESS

---

## The Problem

Current `compiler.rs` BYPASSES the typechecker:

```rust
// compiler.rs compile_llvm() - NO TYPECHECKER!
pub fn compile_llvm(&self, program: &Program) -> Result<String> {
    let processed_program = self.process_imports(program)?;
    let processed_program = self.execute_comptime(processed_program)?;
    let processed_program = self.resolve_self_types(processed_program)?;
    // ❌ MISSING: typechecker.check(&processed_program)?;
    let monomorphized = monomorphizer.monomorphize_program(&processed_program)?;
    llvm_compiler.compile_program(&monomorphized)?;
}
```

Result: Type errors caught late (in codegen) with bad error messages.

---

## Step-by-Step Plan

### Step 1: Audit Current Typechecker ✅ DONE
- [x] Read `typechecker/mod.rs` - entry point is `check_program(&mut self, program: &Program) -> Result<()>`
- [x] Read `compiler.rs` - was missing typechecker call entirely
- [x] Typechecker does 4 passes: collect types, resolve generics, infer returns, check bodies

### Step 2: Add Typechecker Call ✅ DONE
- [x] Added `TypeChecker::new().check_program()` call in `compiler.rs`
- [x] Added to both `compile_llvm()` and `get_module()`
- [x] Placed after `resolve_self_types()`, before `monomorphize()`

### Step 3: Verify No Regression ✅ DONE
- [x] All 23 tests pass
- [x] Build succeeds

### Step 4: Remove Duplicate Type Logic from Codegen ⬅️ CURRENT
**Problem Found:** Two parallel ~1000 LOC inference systems!
- `src/typechecker/inference.rs` (1,008 LOC) - correct location
- `src/codegen/llvm/expressions/inference.rs` (1,023 LOC) - DUPLICATE

**Hardcoded collections in codegen (should be stdlib):**
- `vec_support.rs` (326 LOC) - Fixed-size Vec methods
- `stdlib_codegen/collections.rs` (670 LOC) - HashMap/Set logic

**Tasks:**
- [ ] Audit what codegen inference does vs typechecker inference
- [ ] Remove duplicate logic from codegen
- [ ] Ensure codegen trusts typechecker's results

### Step 5: Intrinsics vs Features Boundary
**Current intrinsics (correct - in `src/intrinsics.rs`):**
- Memory: raw_allocate, raw_deallocate, memcpy, memset
- Pointers: gep, gep_struct, ptr_to_int, int_to_ptr
- Types: sizeof, alignof
- Atomics, syscalls, FFI

**Hardcoded features (WRONG - should be Zen stdlib):**
- HashMap methods (insert, get, remove)
- Vec methods (push, pop, get, set, len)
- Array methods
- String operations

**Principle:** If it can be written in Zen using intrinsics, it should be.

---

## Key Files

| File | Purpose | LOC |
|------|---------|-----|
| `src/compiler.rs` | Pipeline orchestrator | 422 |
| `src/typechecker/mod.rs` | Main typechecker | 991 |
| `src/typechecker/inference.rs` | Type inference | 1,008 |
| `src/codegen/llvm/expressions/inference.rs` | ❌ Duplicate inference | ~400 |

---

## Current Pipeline (FIXED ✅)

```
Source
  ↓
Lexer (lexer.rs)
  ↓
Parser (parser/)
  ↓
process_imports() ← module_system/
  ↓
execute_comptime() ← comptime/
  ↓
resolve_self_types() ← typechecker/self_resolution.rs
  ↓
typechecker.check_program() ← typechecker/ ✅ NEW
  ↓
monomorphize() ← lowering/
  ↓
compile_program() ← codegen/llvm/
  ↓
LLVM IR
```

**Fixed:** Typechecker now integrated into main compilation pipeline!

---

## Target Pipeline

```
Source
  ↓
Lexer
  ↓
Parser
  ↓
═══════════════════════════
  SEMA (semantic analysis)
═══════════════════════════
  ├─ process_imports()
  ├─ execute_comptime()
  ├─ typecheck() ← NEW
  ├─ resolve_self_types()
  └─ monomorphize()
═══════════════════════════
  ↓
Codegen (no type decisions!)
  ↓
LLVM IR
```

---

## Progress Log

### Session: Jan 9, 2026
- [x] Identified typechecker was bypassed in main pipeline
- [x] Created refactor plan
- [x] **Step 1 DONE**: Audited typechecker entry point (`check_program`)
- [x] **Step 2 DONE**: Added typechecker call to `compiler.rs`
- [x] **Step 3 DONE**: All 23 tests pass

### Next Steps
- [x] Step 4: Identified duplicate type inference (see INTRINSICS_BOUNDARY.md)
- [x] Step 5: Deleted `vec_support.rs` - 326 LOC dead code!
- [x] Step 6: Audited `stdlib_codegen/collections.rs` - NOT dead, but WRONG architecture
- [x] Step 7: **DELETED** `collections.rs` (670 LOC) - collections now use stdlib Zen!
- [ ] Step 8: Clean up AST type redundancy (Vec vs FixedArray vs DynVec)
- [ ] Step 9: Add type annotations to AST (enable removing codegen inference - 1,023 LOC)
- [ ] Step 10: Fix hardcoded generics in GenericTypeTracker (hardcodes Option, Result, HashMap, etc.)
- [ ] Step 11: Remove false `#[allow(dead_code)]` markers (165 total, many are false positives)
- [ ] Step 12: Fix duplicate module declarations (lib.rs and main.rs both declare modules)
- [ ] Step 13: Split giant modules (codegen/ 11.6K, lsp/ 12K)

---

## Step 6-7: Collection Implementation Cleanup ✅ DONE

**Problem (FIXED):** HashMap/HashSet/DynVec had TWO implementations.

**What Was Removed:**
1. `stdlib_codegen/collections.rs` (670 LOC) - DELETED
2. Constructor interception in `calls.rs` - REMOVED
3. Method interception in `behaviors.rs` (`try_compile_hashmap_method`) - REMOVED

**What Remains (CORRECT):**
- `stdlib/collections/hashmap.zen` (395 LOC) - Uses intrinsics properly
- `stdlib/collections/set.zen` - Uses intrinsics properly
- `stdlib/vec.zen` - Uses intrinsics properly

**Total Dead Code Removed This Session:**
- `vec_support.rs`: 326 LOC
- `collections.rs`: 670 LOC
- **Total: 996 LOC**

---

## Step 10 Finding: Hardcoded Generics

**Problem:** `codegen/llvm/generics.rs` has `GenericTypeTracker` that hardcodes specific types:
- Result<Ok, Err> → tracks `{prefix}_Ok_Type`, `{prefix}_Err_Type`
- Option<T> → tracks `{prefix}_Some_Type`
- HashMap<K,V> → tracks `{prefix}_Key_Type`, `{prefix}_Value_Type`
- Vec, Array, HashSet similarly hardcoded

**Why This Is Bad:**
1. Not extensible to user-defined generic types
2. Uses fragile string-based naming convention
3. Type knowledge spread across multiple files

**Proper Fix:**
The typechecker should annotate AST with resolved types. Codegen should read annotations, not re-track types.

---

## Step 9 Finding: Duplicate Type Inference

**Problem:** Two parallel ~1000 LOC type inference systems:
- `typechecker/inference.rs` (1,008 LOC)
- `codegen/llvm/expressions/inference.rs` (1,023 LOC)

**Why This Exists:**
- AST doesn't carry type annotations after typechecking
- Codegen must re-infer types to generate correct LLVM IR

**Proper Fix:**
1. Add `resolved_type: Option<AstType>` field to Expression nodes
2. Typechecker fills in resolved_type during type checking
3. Codegen reads resolved_type instead of re-inferring
4. Delete codegen/expressions/inference.rs (1,023 LOC savings)

---

## Notes for Context Recovery

If context is lost, read these files in order:
1. This file (`docs/REFACTOR_SEMA.md`)
2. `src/compiler.rs` - see the pipeline
3. `src/typechecker/mod.rs` - see what typechecker does
4. `docs/ARCHITECTURE.md` - overall goals
