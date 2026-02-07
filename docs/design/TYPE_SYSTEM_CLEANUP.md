# Type System Cleanup Plan

**Status:** Partially complete (Feb 2026)

## Issue 1: String Parsing Instead of AST

### Problem
We're doing manual string parsing with `contains('<')` and `parse_generic_type_string()` when we should be working with `AstType` values directly.

**Bad Pattern:**
```rust
if name.contains('<') {
    let (base_name, type_args) = parse_generic_type_string(name);
    // ... manual manipulation
}
```

**Good Pattern:**
```rust
// Parse once into AST, work with AST
match parse_type_from_string(name) {
    Ok(AstType::Generic { name, type_args }) => {
        // Work with parsed AST type
    }
    Ok(other) => { /* handle */ }
    Err(_) => { /* handle */ }
}
```

### Files to Fix
- ✅ `src/typechecker/inference/identifiers.rs` - FIXED
- ✅ `src/typechecker/inference/enums.rs` - FIXED
- ✅ `src/lsp/completion/context.rs` - FIXED (uses `name_utils::strip_generics()`)
- ✅ `src/lsp/completion/methods.rs` - FIXED (uses `name_utils::strip_generics()`)
- ✅ `src/lsp/type_inference.rs` - FIXED (uses `name_utils::strip_generics()`)
- ✅ `src/lsp/navigation/struct_fields.rs` - FIXED (uses `name_utils::strip_generics()`)
- ✅ `src/lsp/hover/patterns.rs` - FIXED (uses `name_utils::strip_generics()`)
- ✅ `src/lsp/pattern_checking.rs` - FIXED (uses `name_utils::strip_generics()`)
- ⚠️ `src/codegen/llvm/behaviors.rs` - Needs review
- ⚠️ `src/codegen/llvm/expressions/inference.rs` - Needs review

### Root Cause
When identifiers come in as strings (e.g., from LSP or error messages), we parse them manually instead of using the parser once and working with AST.

### Progress (Feb 2026)
Created `src/name_utils.rs` with `strip_generics()`, `method_key()`, `scoped_var_key()`, `stdlib_func_key()` — all LSP string-parsing sites now use these canonical helpers.

---

## Issue 2: Type System Module Duplication

### Current Structure

```
src/
├── typechecker/          # Type checking logic
│   ├── structs: HashMap<String, StructInfo>
│   ├── enums: HashMap<String, EnumInfo>
│   ├── functions: HashMap<String, FunctionSignature>
│   └── stdlib_modules: HashMap<String, Program>
│
├── type_context.rs       # Shared type info (typechecker → codegen)
│   ├── structs: HashMap<String, Vec<(String, AstType)>>
│   ├── enums: HashMap<String, Vec<(String, Option<AstType>)>>
│   └── functions: HashMap<String, FunctionType>
│
├── type_system/          # Generic type resolution
│   ├── environment.rs    # Generic type storage
│   │   ├── generic_functions: HashMap<String, Function>
│   │   ├── generic_structs: HashMap<String, StructDefinition>
│   │   └── generic_enums: HashMap<String, EnumDefinition>
│   ├── monomorphization.rs
│   └── instantiation.rs
│
├── stdlib_types.rs       # Stdlib type registry (mostly unused now)
│   ├── structs: HashMap<String, StructDefinition>
│   ├── methods: HashMap<String, MethodSignature>
│   └── functions: HashMap<String, FunctionSignature>
│
└── well_known.rs         # Registry of well-known types (Option, Result, Ptr)
    └── types: HashMap<String, WellKnownType>
```

### Duplication Analysis

| Data | TypeChecker | TypeContext | TypeEnvironment | StdlibTypes | WellKnown |
|------|-------------|-------------|-----------------|-------------|-----------|
| Structs | ✅ (checking) | ✅ (codegen) | ✅ (generic only) | ✅ (stdlib) | ❌ |
| Enums | ✅ (checking) | ✅ (codegen) | ✅ (generic only) | ❌ | ❌ |
| Functions | ✅ (checking) | ✅ (codegen) | ✅ (generic only) | ✅ (stdlib) | ❌ |
| Methods | ✅ (stdlib) | ✅ (codegen) | ❌ | ✅ (stdlib) | ❌ |
| Well-known types | ❌ | ❌ | ❌ | ❌ | ✅ |

### Problems

1. **TypeChecker and TypeContext duplicate the same data**
   - TypeChecker collects types during checking
   - TypeContext copies them for codegen
   - **Solution**: TypeContext should reference TypeChecker's data or be merged

2. **TypeEnvironment duplicates generic type storage**
   - Stores generic functions/structs/enums separately
   - TypeChecker also tracks these
   - **Solution**: TypeEnvironment should use TypeChecker's data

3. **StdlibTypes is mostly unused**
   - After our refactor, TypeChecker extracts stdlib types directly
   - Only used by LSP now
   - **Solution**: Remove or consolidate into TypeChecker

4. **WellKnown is fine** - It's a registry, not duplication

### Current Architecture (Feb 2026)

TypeStore exists and is partially integrated:

```
src/
├── type_system/
│   ├── type_store.rs     # ✅ CREATED — Unified storage (425 LOC)
│   ├── type_aliases.rs   # ✅ CREATED — Alias resolution with cycle detection (296 LOC)
│   ├── monomorphization.rs
│   ├── instantiation.rs
│   └── environment.rs    # (To be consolidated into TypeStore)
│
├── typechecker/
│   ├── mod.rs            # Uses Rc<RefCell<TypeStore>> for type storage
│   └── ...
│
├── type_context.rs       # Bridge to codegen (still separate)
│
└── well_known.rs         # Registry (keep as-is)
```

### Remaining Migration Steps

1. **Make TypeContext delegate to TypeStore**
   - TypeContext should be a thin view, not a copy
   - Currently still duplicates some data

2. **Remove StdlibTypes duplication**
   - TypeChecker already extracts stdlib types
   - LSP still uses `stdlib_types()` in 16 places — needs incremental migration

3. **Consolidate TypeEnvironment into TypeStore**
   - Don't duplicate generic type storage
   - Query TypeStore for generic types

4. **Single source of truth**
   - All type queries go through TypeStore
   - Other modules reference, don't duplicate
