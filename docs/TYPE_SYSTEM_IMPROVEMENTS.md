# Type System Improvements: From Hardcoded Strings to AST-Based Reality

**Date:** 2026-02-07  
**Status:** In Progress  
**Goal:** Eliminate technical debt, consolidate modules, make type system derive from actual Zen source code

---

## Executive Summary

The Zen type system currently has significant technical debt:
- **15+ locations** use string-based type checking instead of AST
- **3 separate modules** (TypeChecker, TypeContext, TypeEnvironment) duplicate type storage
- **Hardcoded type layouts** in codegen instead of deriving from stdlib source
- **No centralized type alias handling**

This document outlines the path to a clean, AST-based type system that derives its knowledge from actual Zen source code, not hardcoded strings.

---

## Current Problems

### 1. String-Based Type Checking (CRITICAL)

**Problem:** Using string `.contains()` instead of proper type checking

```rust
// BAD: src/lsp/server.rs:315
if expected.contains("Option") && !actual.contains("Option") {
    // This matches "MyOption", "OptionWrapper", "MaybeOption" too!
}

// BAD: src/lsp/server.rs:316  
if expected.contains("Result") && !actual.contains("Result") {
    // Same problem
}
```

**Impact:**
- False positives in type mismatch detection
- LSP shows incorrect hints
- Refactoring tools may break

**Solution:** Use `WellKnownTypes` registry:
```rust
// GOOD:
if well_known().is_option(expected_type_name) {
    // Exact match only
}
```

**Files to fix:**
- `src/lsp/server.rs` (lines 315-367)
- `src/lsp/code_action/mod.rs` (lines 54-66)
- `src/parser/expressions/primary.rs` (lines 59, 102, 107)
- `src/codegen/llvm/types.rs` (line 290)
- `src/lsp/pattern_checking.rs` (line 230)
- `src/lsp/analyzer.rs` (lines 418, 467)

---

### 2. Module Duplication (CRITICAL)

**Problem:** Three modules store the same type information differently

```
src/
├── typechecker/          # Type checking logic
│   ├── structs: HashMap<String, StructInfo>
│   ├── enums: HashMap<String, EnumInfo>
│   └── functions: HashMap<String, FunctionSignature>
│
├── type_context.rs       # Shared type info (typechecker → codegen)
│   ├── structs: HashMap<String, Vec<(String, AstType)>>
│   ├── enums: HashMap<String, Vec<(String, Option<AstType>)>>
│   └── functions: HashMap<String, FunctionType>
│
└── type_system/          # Generic type resolution
    └── environment.rs    # Generic type storage
        ├── generic_functions
        ├── generic_structs
        └── generic_enums
```

**Problems:**
1. TypeChecker and TypeContext **duplicate the same data**
2. TypeEnvironment stores generics separately when TypeChecker already tracks them
3. Conversions between formats are error-prone
4. Memory overhead from multiple copies

**Solution:** Single Source of Truth

```rust
// NEW: src/type_system/type_store.rs
pub struct TypeStore {
    // All types stored once, referenced by other modules
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    functions: HashMap<String, FunctionInfo>,
    type_aliases: HashMap<String, AstType>,
    
    // Separate views (not copies!) for different consumers
    stdlib_methods: HashMap<String, MethodSignature>,
}

// TypeChecker uses TypeStore
pub struct TypeChecker {
    store: Rc<RefCell<TypeStore>>,
    // ... checking state only
}

// TypeContext becomes a thin wrapper
pub struct TypeContext {
    store: Rc<RefCell<TypeStore>>,
}
```

**Migration:**
1. Create `TypeStore` with unified storage
2. Make TypeChecker use TypeStore
3. Make TypeContext delegate to TypeStore
4. Remove TypeEnvironment (use TypeStore directly)

---

### 3. Hardcoded Type Layouts (HIGH)

**Problem:** Codegen hardcodes struct layouts instead of reading from stdlib

```rust
// BAD: src/codegen/llvm/types.rs:290-303
if (name == "Vec" || name == "DynVec") && !type_args.is_empty() {
    let vec_struct = self.context.struct_type(
        &[
            ptr_type.into(),      // data: Ptr<T>
            len_type.into(),      // len: should be ptr_sized_int
            len_type.into(),      // capacity: should be ptr_sized_int
            ptr_type.into(),      // allocator
        ],
        false,
    );
}
```

**Problems:**
- Vec is Layer 3 - should have NO special compiler handling
- Layout assumes 4 fields without validation
- Uses hardcoded i64 instead of platform-sized usize
- Changes to stdlib Vec require compiler updates

**Solution:** Query from stdlib AST

```rust
// GOOD: Query actual stdlib definition
if let Some(struct_info) = type_store.get_struct(name) {
    let field_types: Vec<BasicTypeEnum> = struct_info
        .fields
        .iter()
        .map(|(_, ty)| self.to_llvm_type(ty))
        .collect::<Result<Vec<_>>>()?;
    
    self.context.struct_type(&field_types, false)
}
```

**Files to fix:**
- `src/codegen/llvm/types.rs` (Vec/DynVec hardcoding)
- `src/codegen/llvm/types.rs` (Range type ignores actual types)
- `src/codegen/llvm/mod.rs` (ptr_sized_int hardcoded to i64)

---

### 4. Missing TypeAliasRegistry (HIGH)

**Problem:** No centralized type alias handling

```rust
// Alias handling scattered across 5+ files:
// - lsp/hover/response.rs:152-156
// - lsp/navigation/ufc.rs:148-149
// - parser/types.rs:27
// - ast/types.rs:189
// - lsp/utils.rs:407
```

**Problems:**
- Each file implements its own alias resolution
- Inconsistent behavior
- StaticString/String normalization scattered

**Solution:** Centralized TypeAliasRegistry

```rust
// NEW: src/type_system/type_aliases.rs
pub struct TypeAliasRegistry {
    aliases: HashMap<String, AstType>,
    // Cache for resolved types
    resolved_cache: HashMap<String, AstType>,
}

impl TypeAliasRegistry {
    pub fn register(&mut self, name: &str, target: AstType) {
        self.aliases.insert(name.to_string(), target);
    }
    
    pub fn resolve(&mut self, name: &str) -> Option<AstType> {
        // Handle chains: A -> B -> C
        // Cache results
    }
    
    pub fn normalize(&self, ty: &AstType) -> AstType {
        // Replace aliases with canonical types
    }
}
```

---

### 5. Hardcoded Lists Instead of Source (MEDIUM)

**Problem:** Hardcoded lists instead of scanning stdlib source

```rust
// BAD: src/ast/primitives.rs:288
pub const MATH_FUNCTIONS: &[&str] = &["min", "max", "abs", "sqrt", "pow", "sin", "cos", "tan"];
// Actual stdlib/math.zen has: abs, abs64, factorial, is_even, is_odd, max, min, clamp, fmin, fmax

// BAD: src/ast/primitives.rs:232
pub const COLLECTION_TYPES: &[&str] = &["Vec", "DynVec", "Array", "HashMap", "HashSet"];
// Missing: String, Queue, Stack, LinkedList
```

**Solution:** Parse actual stdlib files

```rust
// GOOD: Scan stdlib at compile time or startup
pub struct StdlibScanner {
    pub fn scan_directory(&mut self, path: &Path) -> Result<StdlibTypes> {
        // Parse all .zen files
        // Extract function names, types, structs
        // Return discovered types
    }
}
```

---

## Implementation Plan

### Phase 1: Foundation (Week 1)

1. **Create TypeStore module**
   - File: `src/type_system/type_store.rs`
   - Unify struct/enum/function storage
   - Provide reference-based access (no copies)

2. **Create TypeAliasRegistry**
   - File: `src/type_system/type_aliases.rs`
   - Centralized alias resolution
   - Handle alias chains and caching

3. **Update WellKnown registry**
   - Add helper methods for common checks
   - Ensure all type comparisons go through WellKnown

### Phase 2: Migration (Week 2)

1. **Migrate TypeChecker to use TypeStore**
   - Update TypeChecker to hold TypeStore reference
   - Remove duplicate storage
   - Update all access patterns

2. **Migrate TypeContext to delegate to TypeStore**
   - Make TypeContext a thin wrapper
   - Ensure codegen still works
   - Remove duplicate data structures

3. **Remove TypeEnvironment**
   - Move functionality to TypeStore
   - Update type_system/ modules

### Phase 3: Cleanup (Week 3)

1. **Remove string-based type checks**
   - Update LSP server
   - Update parser
   - Update codegen

2. **Remove hardcoded layouts**
   - Update LLVM types generation
   - Query from TypeStore instead
   - Fix 64-bit assumptions

3. **Consolidate type alias handling**
   - Replace scattered implementations
   - Use TypeAliasRegistry everywhere

### Phase 4: Verification (Week 4)

1. **Run full test suite**
   - Unit tests
   - Integration tests
   - Behavioral tests

2. **Update documentation**
   - Update ARCHITECTURE.md
   - Update type system design docs
   - Document new APIs

3. **Performance testing**
   - Ensure no regressions
   - Profile memory usage
   - Check compile times

---

## New Module Structure

```
src/
├── type_system/              # Single source of truth for types
│   ├── mod.rs               # Public exports
│   ├── type_store.rs        # Unified type storage
│   ├── type_aliases.rs      # Alias resolution
│   ├── monomorphization.rs  # Generic instantiation
│   └── instantiation.rs     # Type substitution
│
├── typechecker/             # Type checking logic only
│   ├── mod.rs               # TypeChecker uses TypeStore
│   ├── inference/           # Type inference modules
│   ├── validation.rs        # Type compatibility
│   └── ...                  # Other checking modules
│
├── type_context.rs          # Thin wrapper (delegates to TypeStore)
│
└── well_known.rs            # Registry for special types
```

---

## Benefits

### 1. **Maintainability**
- Single source of truth for types
- No duplicate data structures
- Clear separation of concerns

### 2. **Correctness**
- AST-based type checking (no string parsing)
- Type layouts derived from source
- Consistent alias resolution

### 3. **Performance**
- Shared type storage (less memory)
- Cached alias resolution
- No duplicate conversions

### 4. **Cross-Platform**
- Platform-sized integers
- No hardcoded 64-bit assumptions
- Proper DataLayout usage

### 5. **Future-Proof**
- Easy to add new type features
- Self-hosting friendly
- Clean APIs for tools

---

## Success Criteria

- [ ] Zero string-based type checks remaining
- [ ] TypeChecker, TypeContext, TypeEnvironment consolidated
- [ ] All type layouts derived from stdlib source
- [ ] TypeAliasRegistry handles all aliases
- [ ] All tests passing
- [ ] No performance regressions
- [ ] Documentation updated

---

## Notes

- This is a significant refactoring - may take 2-4 weeks
- Changes should be incremental (don't break main)
- Each phase should be independently testable
- Keep backward compatibility during migration
- Consider feature flags for gradual rollout

---

*Document created for the Zen type system refactoring initiative.*
