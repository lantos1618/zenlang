# Zen Compiler Architecture

**Last Updated:** February 2026 (type system refactoring, comptime restructure)

---

## Overview

Zen is a systems programming language with a Rust compiler targeting LLVM. The compiler follows a traditional pipeline architecture with clear phase boundaries.

For language syntax and features, see the [README](../README.md) and [LANGUAGE_SPEC.zen](../LANGUAGE_SPEC.zen).

---

## Compilation Pipeline

```
Source (.zen)
     │
     ▼
┌─────────┐
│  Lexer  │  lexer.rs (780 LOC)
└────┬────┘
     │ Tokens
     ▼
┌─────────┐
│ Parser  │  parser/ (6,798 LOC)
└────┬────┘
     │ AST
     ▼
═══════════════════════════════════
     SEMANTIC ANALYSIS (sema)
═══════════════════════════════════
     │
     ├─► process_imports()     Module resolution
     │
     ├─► execute_comptime()    Compile-time evaluation
     │
     ├─► resolve_self_types()  Self type resolution
     │
     ├─► typecheck()           Type checking & inference
     │       └─► TypeStore     Single source of truth
     │
     └─► monomorphize()        Generic instantiation
═══════════════════════════════════
     │ Typed AST (no generics)
     ▼
┌──────────┐
│ Codegen  │  codegen/ (12,381 LOC)
└────┬─────┘
     │ LLVM IR
     ▼
┌──────────┐
│   LLVM   │  External (Inkwell bindings)
└────┬─────┘
     │
     ▼
  Machine Code
```

---

## Module Structure

```
src/                             183 files, ~55,400 LOC total
├── lib.rs               (18 LOC)    Module exports
├── compiler.rs          (433 LOC)   Pipeline orchestration
├── lexer.rs             (780 LOC)   Tokenization
├── error.rs             (700 LOC)   Error types & helpers
├── well_known.rs        (307 LOC)   Built-in type registry
├── stdlib_types.rs      (423 LOC)   Stdlib type parsing (recursive scanner)
├── stdlib_discovery.rs  (256 LOC)   Stdlib path resolution
├── intrinsics.rs        (217 LOC)   Compiler intrinsics
├── formatting.rs        (594 LOC)   Code formatter
├── name_utils.rs        (141 LOC)   Canonical key construction & parsing
├── type_context.rs      (233 LOC)   Type info bridge (typechecker → codegen)
│
├── ast/                 (1,641 LOC)  Abstract Syntax Tree
│   ├── mod.rs                        Program, node definitions
│   ├── expressions.rs                Expression enum (50+ variants)
│   ├── statements.rs                 Statement enum
│   ├── declarations.rs               Function/struct/enum decls
│   ├── types.rs                      AstType enum
│   ├── patterns.rs                   Pattern matching AST
│   ├── primitives.rs                 Primitive types, constants
│   └── builtins.rs                   Builtin type definitions
│
├── parser/              (6,798 LOC)  Syntax analysis
│   ├── mod.rs                        Parser struct, entry point
│   ├── core.rs                       Token consumption, recursion limits
│   ├── program.rs                    Top-level parsing
│   ├── statements.rs                 Statement parsing
│   ├── statements_guard.rs           Statement boundary detection
│   ├── patterns.rs                   Pattern matching
│   ├── types.rs                      Type annotations
│   ├── functions.rs                  Function declarations
│   ├── structs.rs                    Struct definitions
│   ├── enums.rs                      Enum definitions
│   ├── behaviors.rs                  Behavior definitions
│   ├── comptime.rs                   Comptime block parsing
│   ├── external.rs                   External declarations
│   └── expressions/     (2,530 LOC)  Expression parsing
│       ├── primary.rs                Identifiers, literals
│       ├── operators.rs              Binary/unary ops
│       ├── calls.rs                  Function/method calls
│       ├── control_flow.rs           if/match/while exprs
│       ├── collections.rs            Array/map literals
│       ├── blocks.rs                 Block expressions
│       ├── literals.rs               Literal parsing
│       ├── patterns.rs               Pattern expressions
│       └── structs.rs                Struct literal parsing
│
├── typechecker/         (5,603 LOC)  Type checking
│   ├── mod.rs           (1,220 LOC)  Main typechecker, StructInfo with field index
│   ├── expression_inference.rs       Expression type inference
│   ├── statement_checking.rs         Validate statements
│   ├── declaration_checking.rs       Validate declarations
│   ├── behaviors.rs                  Behavior checking
│   ├── validation.rs                 Type compatibility
│   ├── self_resolution.rs            Self type resolution
│   ├── type_resolution.rs            Resolve type names
│   ├── scope.rs                      Scope management
│   ├── stdlib_loading.rs             Stdlib type loading
│   ├── method_types.rs               Method type inference
│   ├── function_checking.rs          Function body checking
│   ├── pattern_binding.rs            Pattern variable binding
│   ├── types.rs                      Type helper definitions
│   ├── intrinsics.rs                 Intrinsic type checking
│   └── inference/       (1,343 LOC)  Specialized inference
│       ├── mod.rs                    Module exports
│       ├── calls.rs                  Method call resolution (4-phase pipeline)
│       ├── enums.rs                  Enum variant inference
│       ├── member_access.rs          Field access (O(1) via StructInfo index)
│       ├── binary_ops.rs             Binary operation types
│       ├── identifiers.rs            Identifier resolution
│       ├── closures.rs               Closure type inference
│       ├── casts.rs                  Cast validation
│       ├── result_ops.rs             Result/Option operations
│       └── helpers.rs                Shared helpers
│
├── type_system/         (1,671 LOC)  Type storage & monomorphization
│   ├── mod.rs                        Public exports
│   ├── type_store.rs    (425 LOC)    Unified type storage (single source of truth)
│   ├── type_aliases.rs  (296 LOC)    Alias resolution with cycle detection
│   ├── monomorphization.rs           Generic instantiation
│   ├── instantiation.rs              Type substitution
│   └── environment.rs                Type environment
│
├── codegen/             (12,381 LOC) LLVM backend
│   └── llvm/
│       ├── mod.rs       (860 LOC)    LLVMCompiler struct
│       ├── types.rs                  AstType → LLVM type
│       ├── symbols.rs                Symbol table
│       ├── behaviors.rs              Behavior dispatch
│       ├── generics.rs               Generic tracking
│       ├── binary_ops.rs             Arithmetic/logic ops
│       ├── literals.rs               Literal codegen
│       ├── patterns.rs               Pattern matching
│       ├── structs.rs                Struct layout
│       ├── pointers.rs               Pointer ops
│       ├── builtins.rs               Builtin operations
│       ├── functions/   (1,154 LOC)
│       │   ├── decl.rs               Function declarations
│       │   ├── calls.rs              Call site codegen
│       │   └── mod.rs                Module exports
│       ├── expressions/ (3,673 LOC)
│       │   ├── inference.rs          Type inference (~1,100 LOC)
│       │   ├── utils.rs              Utilities (~970 LOC)
│       │   ├── enums.rs              Enum variants
│       │   ├── control.rs            If/match codegen
│       │   ├── patterns.rs           Pattern codegen
│       │   ├── calls.rs              Call codegen
│       │   ├── collections.rs        Collection ops
│       │   ├── structs.rs            Struct expressions
│       │   ├── literals.rs           Literal expressions
│       │   ├── operations.rs         Operations
│       │   └── mod.rs                Module exports
│       ├── statements/  (897 LOC)
│       │   ├── variables.rs          Variable decl/assign
│       │   ├── control.rs            Return/loop/break
│       │   ├── deferred.rs           Defer execution
│       │   └── mod.rs                Module exports
│       └── stdlib_codegen/ (1,476 LOC)
│           ├── compiler.rs           Intrinsic implementations
│           ├── helpers.rs            Codegen helpers
│           └── mod.rs                Module exports
│
├── lsp/                 (16,399 LOC) Language Server
│   ├── server.rs        (1,080 LOC)  Main server loop, request routing
│   ├── mod.rs                        Constants, search limits
│   ├── types.rs                      Document, SymbolInfo types
│   ├── analyzer.rs                   Background analysis coordination
│   ├── utils.rs                      Shared utilities
│   ├── helpers.rs                    Helper functions
│   ├── compiler_integration.rs       TypeChecker bridge
│   ├── stdlib_resolver.rs            Stdlib symbol resolution
│   ├── symbol_extraction.rs          Symbol extraction
│   ├── semantic_completion.rs        TypeContext-based completion
│   ├── type_inference.rs             LSP type inference
│   ├── pattern_checking.rs           Pattern completeness
│   ├── signature_help.rs             Function signatures
│   ├── inlay_hints.rs                Inline type hints
│   ├── semantic_tokens.rs            Syntax highlighting
│   ├── rename.rs                     Symbol renaming
│   ├── code_lens.rs                  Run/Build/Test buttons
│   ├── call_hierarchy.rs             Call tree
│   ├── symbols.rs                    Document/workspace symbols
│   ├── formatting.rs                 Code formatting
│   ├── indexing.rs                   Symbol indexing
│   ├── document_store/  (1,277 LOC)  Open document management
│   │   ├── mod.rs                    Store struct, lifecycle
│   │   ├── symbol_extraction.rs      Extract symbols from AST
│   │   ├── builtin_registration.rs   Register stdlib symbols
│   │   ├── symbol_search.rs          Symbol search
│   │   ├── document_lifecycle.rs     Open/close/update
│   │   ├── variable_extraction.rs    Extract variable info
│   │   ├── reference_tracking.rs     Reference tracking
│   │   ├── utilities.rs              Utility functions
│   │   └── parsing.rs                Document parsing
│   ├── completion/      (1,195 LOC)  Code completion
│   │   ├── mod.rs                    Completion dispatcher
│   │   ├── context.rs               Context analysis
│   │   ├── methods.rs               Method completions
│   │   ├── auto_import.rs           Auto-import support
│   │   └── modules.rs              Module completions
│   ├── hover/           (2,246 LOC)  Hover information
│   │   ├── mod.rs       (832 LOC)   Main dispatcher
│   │   ├── expressions.rs           Expression hover
│   │   ├── patterns.rs              Pattern hover
│   │   ├── builtins.rs              Builtin hover
│   │   ├── format_string.rs         Format string hover
│   │   ├── structs.rs               Struct hover
│   │   ├── inference.rs             Type inference hover
│   │   ├── response.rs              Response formatting
│   │   └── imports.rs               Import hover
│   ├── navigation/      (2,288 LOC)  Navigation features
│   │   ├── definition.rs (818 LOC)  Go-to-definition
│   │   ├── struct_fields.rs         Struct field navigation
│   │   ├── references.rs            Find references
│   │   ├── ufc.rs                   UFC navigation
│   │   ├── utils.rs                 Navigation utilities
│   │   ├── type_definition.rs       Type definition
│   │   ├── highlight.rs             Document highlight
│   │   ├── scope.rs                 Scope navigation
│   │   ├── imports.rs               Import navigation
│   │   └── mod.rs                   Module exports
│   └── code_action/     (1,272 LOC)  Quick fixes & refactoring
│       ├── refactorings.rs          Refactoring actions
│       ├── quick_fixes.rs           Quick fix suggestions
│       ├── imports.rs               Import fixes
│       ├── mod.rs                   Action dispatcher
│       ├── utils.rs                 Utility functions
│       └── suggestions.rs           Code suggestions
│
├── comptime/            (4,020 LOC)  Compile-time evaluation
│   ├── mod.rs           (871 LOC)   Interpreter core, with_scope, control flow
│   ├── expressions.rs               Expression evaluation
│   ├── statements.rs                Statement evaluation
│   ├── methods.rs                   Method call evaluation
│   ├── values.rs                    ComptimeValue + Display
│   ├── environment.rs               Variable environment
│   └── meta/            (1,518 LOC) AST introspection
│       ├── mod.rs                   Meta API entry point
│       ├── fields.rs                AST field extraction
│       ├── helpers.rs               Shared builders
│       ├── variants.rs              Variant name constants
│       └── tests.rs                 Meta tests
│
├── module_system/       (520 LOC)   Module resolution
│   ├── mod.rs                       Module registry
│   └── resolver.rs                  Import resolution
│
└── bin/                 (406 LOC)
    ├── zen-lsp.rs                   LSP server binary
    ├── zen-format.rs                Formatter binary
    └── zen-check.rs                 Checker binary
```

---

## Metrics

| Metric | Value |
|--------|-------|
| Total Rust files | 183 |
| Total LOC | ~55,400 |
| Test count (lib) | 143 |

### Module Sizes

| Module | LOC | Notes |
|--------|-----|-------|
| lsp/ | 16,399 | Full LSP implementation |
| codegen/ | 12,381 | LLVM backend |
| parser/ | 6,798 | Syntax analysis |
| typechecker/ | 5,603 | Type checking & inference |
| comptime/ | 4,020 | Compile-time evaluation + meta API |
| type_system/ | 1,671 | TypeStore, aliases, monomorphization |
| ast/ | 1,641 | AST definitions |
| lexer.rs | 780 | Single-file tokenizer |
| module_system/ | 520 | Module resolution |

---

## Key Architecture Concepts

### TypeStore (Single Source of Truth)

`src/type_system/type_store.rs` is the unified type storage used by the TypeChecker. All struct, enum, function, method, and variable type information flows through TypeStore.

```rust
pub struct TypeStore {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    functions: HashMap<String, FunctionType>,
    methods: HashMap<String, AstType>,
    variables: HashMap<String, AstType>,
    // ...
}
```

The TypeChecker holds `Rc<RefCell<TypeStore>>` and populates it during analysis. TypeContext then provides a read-only view for codegen.

### name_utils (Canonical Key Construction)

`src/name_utils.rs` provides canonical key construction functions to eliminate ad-hoc string formatting:

| Function | Format | Example |
|----------|--------|---------|
| `method_key(type, method)` | `"Type.method"` | `"Vec.len"` |
| `scoped_var_key(scope, var)` | `"scope::var"` | `"main::x"` |
| `stdlib_func_key(module, func)` | `"module::func"` | `"io::println"` |
| `strip_generics(name)` | Base name only | `"Vec<i32>"` → `"Vec"` |

All method keys use `"."` separator (unified from mixed `.`/`::` formats).

### StructInfo with Field Index

`StructInfo` (in `typechecker/mod.rs`) provides O(1) field lookups via a lazy `HashMap` index:

```rust
pub struct StructInfo {
    pub fields: Vec<(String, AstType)>,
    field_index: Option<HashMap<String, usize>>,
}

impl StructInfo {
    pub fn get_field_type(&mut self, name: &str) -> Option<&AstType>;
    pub fn has_field(&mut self, name: &str) -> bool;
}
```

The index is built on first access for structs with >4 fields.

---

## Intrinsics vs Stdlib

The compiler provides minimal intrinsics. Everything else is in the Zen stdlib.

**Intrinsics** (in compiler, cannot be written in Zen):
- Memory: `raw_allocate`, `raw_deallocate`, `memcpy`, `memset`
- Pointers: `gep`, `gep_struct`, `ptr_to_int`, `int_to_ptr`
- Types: `sizeof<T>`, `alignof<T>`
- Atomics: `atomic_load`, `atomic_store`, `atomic_cas`, etc.
- Syscalls: `syscall0` - `syscall6`
- Enums: `discriminant`, `get_payload`, `set_payload`

**Stdlib** (written in Zen using intrinsics):
- All collections, memory allocators, sync primitives, I/O

See `docs/INTRINSICS_REFERENCE.md` for full intrinsics documentation.

---

## Phase Responsibilities

### Lexer (`lexer.rs`)
- Converts source text to tokens
- No semantic analysis
- Reports lexical errors

### Parser (`parser/`)
- Builds AST from tokens
- No type checking
- Recursion depth limiting (MAX_RECURSION_DEPTH = 256)
- Error recovery for LSP (partial AST on syntax errors)
- Reports syntax errors

### Typechecker (`typechecker/`)
- Type inference and checking via TypeStore
- Behavior implementation verification
- Self type resolution
- Method call resolution (4-phase pipeline in `inference/calls.rs`)
- O(1) struct field lookups via StructInfo index
- Reports type errors

### Monomorphizer (`type_system/`)
- Instantiates generic types with concrete types
- Creates specialized versions of generic functions
- No type inference (trusts typechecker)

### Comptime (`comptime/`)
- Compile-time expression evaluation
- AST introspection via meta API (`@type_info`, `@fields`, etc.)
- Code generation via `emit()` builtin
- Proper control flow (Break/Continue/Return enum, not error strings)
- Scoped environment via `with_scope()` RAII pattern

### Codegen (`codegen/`)
- Generates LLVM IR from typed AST
- No type decisions (trusts previous phases)
- Implements intrinsics

---

## Standard Library Structure

```
stdlib/
├── std.zen             Entry point, re-exports
├── build.zen           Build system
├── compiler.zen        Compiler intrinsics
├── ffi.zen             Foreign function interface
├── math.zen            Math functions
├── testing.zen         Test framework
├── time.zen            Time operations
│
├── core/               Core types
│   ├── option.zen      Option<T>: Some, None
│   ├── result.zen      Result<T,E>: Ok, Err
│   ├── ptr.zen         Ptr<T>, MutPtr<T>, RawPtr<T>
│   ├── iterator.zen    Range, Iterator behavior
│   └── propagate.zen   Error propagation
│
├── collections/        All container types
│   ├── string.zen      Dynamic UTF-8 string
│   ├── vec.zen         Dynamic array Vec<T>
│   ├── char.zen        Character utilities
│   ├── hashmap.zen     HashMap<K,V>
│   ├── set.zen         Set<T>
│   ├── stack.zen       Stack<T>
│   ├── queue.zen       Queue<T>
│   └── linkedlist.zen  LinkedList<T>
│
├── memory/             Memory management
│   ├── allocator.zen   Allocator behavior
│   ├── gpa.zen         General purpose allocator
│   ├── mmap.zen        Memory-mapped regions
│   ├── async_allocator.zen  Async allocator behavior
│   └── async_pool.zen  io_uring-based allocator
│
├── concurrency/        ALL concurrency in one place
│   ├── primitives/     Low-level (atomic, futex)
│   ├── sync/           Thread-based (mutex, channel, thread, etc.)
│   ├── async/          Task-based (task, executor, scheduler)
│   └── actor/          Actor model (actor, supervisor, system)
│
├── io/                 I/O operations
│   ├── io.zen          Basic print/read
│   ├── files/          File ops (file, fs, dir, stat, link, copy, splice)
│   ├── net/            Networking (socket, unix_socket, pipe)
│   └── mux/            I/O multiplexing (epoll, poll, uring)
│
└── sys/                System interface
    ├── syscall.zen     Syscall numbers
    ├── process/        Process management (process, prctl, sched)
    ├── random/         Random (getrandom, prng)
    └── ...             (env, uname, resource, seccomp, memfd)
```

---

## LSP Features

The language server (`src/lsp/`) implements full LSP support:

- Hover with type info
- Go-to-definition (including nested member access)
- Find all references
- Code completion (`.`, `:`, `@`, `?` triggers)
- Signature help
- Document/workspace symbols
- Rename with prepare
- Folding ranges
- Inlay hints
- Call hierarchy
- Semantic tokens
- Document formatting
- Code actions (quick fixes, refactorings, import management)
- Code lens (Run/Build/Test)

---

## Build & Test

```bash
# Build
cargo build --release

# Run all tests
cargo test

# Run compiler
./target/release/zen examples/hello.zen

# Run LSP
./target/release/zen-lsp
```

---

## Compiler Internals

### Well-Known Types

The compiler has special knowledge of these types (`src/well_known.rs`):

| Type | Special Handling |
|------|------------------|
| `Option<T>` | Pattern matching codegen, `?` operator |
| `Result<T,E>` | Pattern matching codegen, `?` operator |
| `Vec<T>` | Indexing, iteration |
| `String` | String interpolation, literals |
| `HashMap<K,V>` | Iteration |
| `Range` | Loop codegen |

### Key Data Structures

**AST Types** (`src/ast/`):
```rust
pub enum Expression {
    Integer32(i32),
    BinaryOp { left, op, right },
    FunctionCall { name, args, generics },
    StructLiteral { name, fields },
    Match { value, arms },
    // ... 50+ variants
}

pub enum Statement {
    Let { name, type_annotation, value },
    Return(Option<Expression>),
    If { condition, then_block, else_block },
    While { condition, body },
    // ...
}
```

**Compiler State** (`src/codegen/llvm/mod.rs`):
```rust
pub struct LLVMCompiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    variables: HashMap<String, VariableInfo>,
    functions: HashMap<String, FunctionValue>,
    struct_types: HashMap<String, StructTypeInfo>,

    generic_tracker: GenericTypeTracker,
    well_known: WellKnownTypes,
}
```

### Extension Points

**Adding a new intrinsic:**
1. Declare in `src/intrinsics.rs`
2. Add codegen in `src/codegen/llvm/stdlib_codegen/compiler.rs`
3. Document in `docs/INTRINSICS_REFERENCE.md`

**Adding a new AST node:**
1. Add variant to `src/ast/expressions.rs` or `src/ast/statements.rs`
2. Add parsing in `src/parser/`
3. Add type checking in `src/typechecker/`
4. Add codegen in `src/codegen/llvm/expressions/` or `statements/`

**Adding stdlib functionality:**
1. Write in Zen using existing intrinsics (`stdlib/*.zen`)
2. No compiler changes needed

---

## Related Documentation

- `docs/INTRINSICS_REFERENCE.md` - Compiler intrinsics reference
- `docs/ROADMAP.md` - Development roadmap
- `docs/design/STDLIB_DESIGN.md` - Stdlib API design
- `docs/design/TYPE_SYSTEM_CLEANUP.md` - Type system cleanup plan
- `docs/design/SEPARATION_OF_CONCERNS.md` - Three-layer architecture
- `docs/QUICK_START.md` - Getting started guide
