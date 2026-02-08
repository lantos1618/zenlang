# Zen Compiler Architecture

**Last Updated:** February 2026

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
│  Lexer  │  lexer.rs
└────┬────┘
     │ Tokens
     ▼
┌─────────┐
│ Parser  │  parser/
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
│ Codegen  │  codegen/
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

## Source Tree with LOC

```
src/                                53,949 LOC total
├── lib.rs                              18
├── main.rs                          1,873
├── compiler.rs                        436   Pipeline orchestration
├── lexer.rs                           780   Tokenization
├── error.rs                           700   Error types & helpers
├── well_known.rs                      307   Built-in type registry
├── stdlib_types.rs                    423   Stdlib type parsing (recursive scanner)
├── stdlib_discovery.rs                243   Stdlib path resolution
├── intrinsics.rs                      226   Compiler intrinsics
├── formatting.rs                      594   Code formatter
├── name_utils.rs                      207   Canonical key construction & parsing
├── type_context.rs                    278   Type info bridge (typechecker → codegen)
│
├── ast/                             2,846
│   ├── mod.rs                          63   Program, node definitions
│   ├── expressions.rs                 742   Expression enum (50+ variants)
│   ├── statements.rs                  244   Statement enum
│   ├── declarations.rs                330   Function/struct/enum decls
│   ├── types.rs                       452   AstType enum
│   ├── fields.rs                      385   Field definitions
│   ├── patterns.rs                    138   Pattern matching AST
│   ├── primitives.rs                  408   Primitive types, constants
│   └── builtins.rs                     84   Builtin type definitions
│
├── parser/                          7,138
│   ├── mod.rs                          85   Parser struct, entry point
│   ├── core.rs                        325   Token consumption, recursion limits
│   ├── program.rs                     386   Top-level parsing
│   ├── statements.rs                1,307   Statement parsing + error recovery
│   ├── statements_guard.rs            238   Statement boundary detection
│   ├── patterns.rs                    440   Pattern matching
│   ├── types.rs                       346   Type annotations
│   ├── functions.rs                   151   Function declarations
│   ├── structs.rs                     189   Struct definitions
│   ├── enums.rs                        91   Enum definitions
│   ├── behaviors.rs                   508   Behavior definitions
│   ├── comptime.rs                    125   Comptime block parsing
│   ├── external.rs                     85   External declarations
│   └── expressions/                 2,511
│       ├── mod.rs                      28
│       ├── primary.rs                 799   Identifiers, literals
│       ├── operators.rs               184   Binary/unary ops
│       ├── calls.rs                   300   Function/method calls
│       ├── control_flow.rs            165   if/match/while exprs
│       ├── collections.rs             272   Array/map literals
│       ├── blocks.rs                   83   Block expressions
│       ├── literals.rs                292   Literal parsing
│       ├── patterns.rs                321   Pattern expressions
│       └── structs.rs                  67   Struct literal parsing
│
├── typechecker/                     5,866
│   ├── mod.rs                       1,373   Main typechecker, StructInfo with field index
│   ├── expression_inference.rs        462   Expression type inference
│   ├── statement_checking.rs          305   Validate statements
│   ├── declaration_checking.rs        312   Validate declarations
│   ├── behaviors.rs                   457   Behavior checking
│   ├── validation.rs                  625   Type compatibility
│   ├── self_resolution.rs             173   Self type resolution
│   ├── type_resolution.rs             146   Resolve type names
│   ├── scope.rs                       169   Scope management
│   ├── stdlib_loading.rs              171   Stdlib type loading
│   ├── method_types.rs                175   Method type inference
│   ├── function_checking.rs            71   Function body checking
│   ├── pattern_binding.rs             144   Pattern variable binding
│   ├── types.rs                       109   Type helper definitions
│   ├── intrinsics.rs                   26   Intrinsic type checking
│   └── inference/                   1,148
│       ├── mod.rs                      22
│       ├── calls.rs                   421   Method call resolution (4-phase pipeline)
│       ├── enums.rs                   247   Enum variant inference
│       ├── member_access.rs           193   Field access (O(1) via StructInfo index)
│       ├── binary_ops.rs              204   Binary operation types
│       ├── identifiers.rs              81   Identifier resolution
│       ├── closures.rs                 86   Closure type inference
│       ├── casts.rs                    44   Cast validation
│       ├── result_ops.rs               25   Result/Option operations
│       └── helpers.rs                  20   Shared helpers
│
├── type_system/                     1,671
│   ├── mod.rs                         128   Public exports
│   ├── type_store.rs                  425   Unified type storage (single source of truth)
│   ├── type_aliases.rs                296   Alias resolution with cycle detection
│   ├── monomorphization.rs            428   Generic instantiation
│   ├── instantiation.rs               288   Type substitution
│   └── environment.rs                 106   Type environment
│
├── codegen/                        12,576
│   ├── mod.rs                           5
│   └── llvm/
│       ├── mod.rs                     860   LLVMCompiler struct
│       ├── types.rs                   537   AstType → LLVM type
│       ├── symbols.rs                 209   Symbol table
│       ├── behaviors.rs               739   Behavior dispatch
│       ├── generics.rs                138   Generic tracking
│       ├── binary_ops.rs              676   Arithmetic/logic ops
│       ├── literals.rs                478   Literal codegen
│       ├── patterns.rs                443   Pattern matching
│       ├── structs.rs                 729   Struct layout
│       ├── pointers.rs                250   Pointer ops
│       ├── builtins.rs                125   Builtin operations
│       ├── functions/               1,154
│       │   ├── mod.rs                  63
│       │   ├── decl.rs                409   Function declarations
│       │   └── calls.rs               682   Call site codegen
│       ├── expressions/             3,673
│       │   ├── mod.rs                 185
│       │   ├── inference.rs         1,110   Type inference
│       │   ├── utils.rs               972   Utilities
│       │   ├── enums.rs               443   Enum variants
│       │   ├── control.rs             291   If/match codegen
│       │   ├── patterns.rs            363   Pattern codegen
│       │   ├── calls.rs               151   Call codegen
│       │   ├── collections.rs          38   Collection ops
│       │   ├── structs.rs              48   Struct expressions
│       │   ├── literals.rs             53   Literal expressions
│       │   └── operations.rs           19   Operations
│       ├── statements/                897
│       │   ├── mod.rs                  49
│       │   ├── variables.rs           631   Variable decl/assign
│       │   ├── control.rs             180   Return/loop/break
│       │   └── deferred.rs             37   Defer execution
│       └── stdlib_codegen/          1,480
│           ├── mod.rs                  70
│           ├── compiler.rs          1,338   Intrinsic implementations
│           └── helpers.rs              72   Codegen helpers
│
├── lsp/                            15,028
│   ├── mod.rs                          66   Constants, search limits
│   ├── server.rs                    1,080   Main server loop, request routing
│   ├── types.rs                       117   Document, SymbolInfo types
│   ├── helpers.rs                     310   Response helpers, param parsing
│   ├── analyzer.rs                    232   Background analysis coordination
│   ├── utils.rs                       705   Shared utilities
│   ├── type_query.rs                  304   TypeContext facade for LSP consumers
│   ├── stdlib_resolver.rs             224   Stdlib symbol resolution
│   ├── symbol_extraction.rs           403   Symbol extraction
│   ├── semantic_completion.rs         349   TypeContext-based completion
│   ├── pattern_checking.rs             65   Pattern completeness
│   ├── signature_help.rs             331   Function signatures
│   ├── inlay_hints.rs                 604   Inline type hints
│   ├── semantic_tokens.rs             368   Syntax highlighting
│   ├── rename.rs                      603   Symbol renaming
│   ├── code_lens.rs                   177   Run/Build/Test buttons
│   ├── call_hierarchy.rs              390   Call tree
│   ├── symbols.rs                     175   Document/workspace symbols
│   ├── formatting.rs                   63   Code formatting
│   ├── indexing.rs                     96   Symbol indexing
│   ├── document_store/              1,119
│   │   ├── mod.rs                     200   Store struct, lifecycle
│   │   ├── parsing.rs                  69   Document parsing
│   │   ├── symbol_extraction.rs       184   Extract symbols from AST
│   │   ├── builtin_registration.rs     97   Register stdlib symbols
│   │   ├── symbol_search.rs           153   Symbol search
│   │   ├── document_lifecycle.rs      137   Open/close/update
│   │   ├── variable_extraction.rs     102   Extract variable info
│   │   ├── reference_tracking.rs      101   Reference tracking
│   │   └── utilities.rs                76   Utility functions
│   ├── completion/                  1,178
│   │   ├── mod.rs                     388   Completion dispatcher
│   │   ├── context.rs                 570   Context analysis
│   │   ├── auto_import.rs             120   Auto-import support
│   │   ├── methods.rs                  46   Method completions
│   │   └── modules.rs                  54   Module completions
│   ├── hover/                       1,898
│   │   ├── mod.rs                     656   Main dispatcher
│   │   ├── patterns.rs                279   Pattern hover
│   │   ├── builtins.rs                251   Builtin hover
│   │   ├── format_string.rs           243   Format string hover
│   │   ├── expressions.rs             177   Expression hover
│   │   ├── response.rs                169   Response formatting
│   │   ├── structs.rs                  66   Struct hover
│   │   ├── inference.rs                53   Type inference hover
│   │   └── imports.rs                   4   Import hover
│   ├── navigation/                  1,847
│   │   ├── mod.rs                      20
│   │   ├── definition.rs              523   Go-to-definition (decomposed resolvers)
│   │   ├── references.rs              272   Find references
│   │   ├── struct_fields.rs           258   Struct field navigation
│   │   ├── ufc.rs                     223   UFC navigation
│   │   ├── utils.rs                   303   Navigation utilities
│   │   ├── type_definition.rs          85   Type definition
│   │   ├── scope.rs                    64   Scope navigation
│   │   ├── imports.rs                  60   Import navigation
│   │   └── highlight.rs                39   Document highlight
│   └── code_action/                 1,272
│       ├── mod.rs                     146   Action dispatcher
│       ├── refactorings.rs            366   Refactoring actions
│       ├── quick_fixes.rs             290   Quick fix suggestions
│       ├── imports.rs                 273   Import fixes
│       ├── utils.rs                   106   Utility functions
│       └── suggestions.rs             91   Code suggestions
│
├── comptime/                        3,127
│   ├── mod.rs                         871   Interpreter core, with_scope, control flow
│   ├── expressions.rs                 444   Expression evaluation
│   ├── statements.rs                  328   Statement evaluation
│   ├── methods.rs                     476   Method call evaluation
│   ├── values.rs                      316   ComptimeValue + Display
│   ├── environment.rs                  67   Variable environment
│   └── meta/                          625
│       ├── mod.rs                     140   Meta API entry point
│       ├── tests.rs                   204   Meta tests
│       ├── variants.rs                177   Variant name constants
│       ├── helpers.rs                  72   Shared builders
│       └── fields.rs                   32   AST field extraction
│
├── module_system/                     597
│   ├── mod.rs                         415   Module registry
│   └── resolver.rs                    182   Import resolution
│
└── bin/                               406
    ├── zen-format.rs                  261   Formatter binary
    ├── zen-check.rs                   133   Checker binary
    └── zen-lsp.rs                      12   LSP server binary
```

```
stdlib/                             13,053 LOC total
├── std.zen                             75   Entry point, re-exports
├── build.zen                          275   Build system
├── compiler.zen                       251   Compiler intrinsics
├── ffi.zen                             99   Foreign function interface
├── math.zen                            67   Math functions
├── testing.zen                        165   Test framework
├── time.zen                           113   Time operations
│
├── core/                              651
│   ├── option.zen                      52   Option<T>: Some, None
│   ├── result.zen                      66   Result<T,E>: Ok, Err
│   ├── ptr.zen                        127   Ptr<T>, MutPtr<T>, RawPtr<T>
│   ├── iterator.zen                    58   Range, Iterator behavior
│   ├── slice.zen                      234   Slice<T>
│   ├── buffer.zen                      94   Buffer
│   └── propagate.zen                   20   Error propagation
│
├── collections/                     2,117
│   ├── string.zen                     338   Dynamic UTF-8 string
│   ├── vec.zen                        224   Dynamic array Vec<T>
│   ├── hashmap.zen                    463   HashMap<K,V>
│   ├── linkedlist.zen                 357   LinkedList<T>
│   ├── stack.zen                      287   Stack<T>
│   ├── set.zen                        196   Set<T>
│   ├── queue.zen                      133   Queue<T>
│   └── char.zen                       119   Character utilities
│
├── memory/                            439
│   ├── allocator.zen                  106   Allocator behavior
│   ├── heap.zen                       117   Heap allocator
│   ├── arena.zen                      113   Arena allocator
│   ├── mmap.zen                        62   Memory-mapped regions
│   ├── async_helpers.zen               59   Async helpers
│   └── async_allocator.zen             42   Async allocator behavior
│
├── concurrency/                     3,354
│   ├── primitives/
│   │   ├── atomic.zen                 204   Atomic operations
│   │   └── futex.zen                   78   Futex
│   ├── sync/
│   │   ├── channel.zen                282   Channel
│   │   ├── thread.zen                 187   Thread
│   │   ├── waitgroup.zen              182   WaitGroup
│   │   ├── rwlock.zen                 168   RWLock
│   │   ├── once.zen                   131   Once
│   │   ├── semaphore.zen              130   Semaphore
│   │   ├── condvar.zen                130   CondVar
│   │   ├── mutex.zen                  114   Mutex
│   │   └── barrier.zen                 64   Barrier
│   ├── async/
│   │   ├── scheduler.zen              338   Scheduler
│   │   └── task.zen                   279   Task
│   └── actor/
│       ├── supervisor.zen             295   Supervisor
│       ├── async_actor.zen            295   Async actor
│       ├── actor.zen                  214   Actor
│       └── system.zen                 203   System
│
├── io/                              2,788
│   ├── io.zen                          73   Basic print/read
│   ├── terminal.zen                   165   Terminal
│   ├── signal.zen                      62   Signal handling
│   ├── eventfd.zen                     64   EventFD
│   ├── inotify.zen                     49   Inotify
│   ├── timerfd.zen                     49   TimerFD
│   ├── files/
│   │   ├── file.zen                   456   File operations
│   │   ├── dir.zen                    246   Directory operations
│   │   ├── stat.zen                   246   File status
│   │   ├── fs.zen                     242   Filesystem operations
│   │   ├── splice.zen                 240   Splice/sendfile
│   │   ├── copy.zen                   179   File copy
│   │   └── link.zen                   175   Hard/symlinks
│   ├── net/
│   │   ├── socket.zen                 415   TCP/UDP sockets
│   │   ├── unix_socket.zen            398   Unix domain sockets
│   │   └── pipe.zen                    45   Pipes
│   └── mux/
│       ├── uring.zen                  464   io_uring
│       ├── poll.zen                     66   Poll
│       └── epoll.zen                   65   Epoll
│
└── sys/                             1,454
    ├── syscall.zen                    221   Syscall numbers
    ├── seccomp.zen                    275   Seccomp filters
    ├── memfd.zen                      268   Memory FDs
    ├── resource.zen                   229   Resource limits
    ├── process/
    │   ├── prctl.zen                  255   Process control
    │   ├── sched.zen                  194   Scheduling
    │   └── process.zen                 95   Process management
    ├── random/
    │   ├── prng.zen                    85   PRNG
    │   └── getrandom.zen               46   Getrandom
    ├── uname.zen                       43   System info
    └── env.zen                         37   Environment variables
```

```
tests/                               4,395 LOC total
├── lsp_analysis_tests.rs            1,004
├── behavioral_tests.rs                944
├── lsp_text_edit.rs                   274
├── lsp_completion_tests.rs            270
├── lsp_code_action_tests.rs           263
├── ptr_ref_tests.rs                   263
├── lsp_navigation_tests.rs            243
├── codegen_integration.rs             240
├── allocator_compilation.rs           238
├── parser_tests.rs                    227
├── lexer_integration.rs               227
├── common/mod.rs                      148
└── lexer_tests.rs                      54
```

---

## Metrics

| Metric | Value |
|--------|-------|
| Compiler source | 53,949 LOC |
| Standard library | 13,053 LOC |
| Tests | 4,395 LOC |
| **Total** | **71,397 LOC** |
| Lib unit tests | 146 |

---

## Key Architecture Concepts

### TypeStore (Single Source of Truth)

`src/type_system/type_store.rs` — unified type storage used by the TypeChecker. All struct, enum, function, method, and variable type information flows through TypeStore.

The TypeChecker holds `Rc<RefCell<TypeStore>>` and populates it during analysis. TypeContext then provides a read-only view for codegen and LSP.

### TypeQuery (LSP → SEMA Bridge)

`src/lsp/type_query.rs` — thin facade over TypeContext for LSP consumers:

```
LSP Request → Parser → AST → TypeChecker → TypeContext → TypeQuery → LSP Response
```

Key methods: `find_variable_type()`, `resolve_chain()`, `has_struct()`, `function_return_type_ast()`, `infer_literal_type()`.

### name_utils (Canonical Key Construction)

`src/name_utils.rs` — eliminates ad-hoc string formatting:

| Function | Format | Example |
|----------|--------|---------|
| `method_key(type, method)` | `"Type.method"` | `"Vec.len"` |
| `scoped_var_key(scope, var)` | `"scope::var"` | `"main::x"` |
| `stdlib_func_key(module, func)` | `"module::func"` | `"io::println"` |
| `strip_generics(name)` | Base name only | `"Vec<i32>"` → `"Vec"` |

All method keys use `"."` separator (unified from mixed `.`/`::` formats).

### StructInfo with Field Index

`StructInfo` (in `typechecker/mod.rs`) provides O(1) field lookups via a lazy `HashMap` index, built on first access for structs with >4 fields.

---

## Phase Responsibilities

| Phase | Module | Responsibility |
|-------|--------|----------------|
| Lexer | `lexer.rs` | Source text → tokens. No semantic analysis. |
| Parser | `parser/` | Tokens → AST. Recursion depth limiting (256). Error recovery for LSP (partial AST on syntax errors). |
| Typechecker | `typechecker/` | Type inference/checking via TypeStore. Behavior verification. Self type resolution. Method call resolution (4-phase pipeline). O(1) struct field lookups. |
| Monomorphizer | `type_system/` | Instantiate generic types with concrete types. No type inference (trusts typechecker). |
| Comptime | `comptime/` | Compile-time expression evaluation. AST introspection via meta API. Code generation via `emit()`. Scoped environment via `with_scope()` RAII. |
| Codegen | `codegen/` | Typed AST → LLVM IR. No type decisions (trusts previous phases). Implements intrinsics. |

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

## LSP Features

Hover, go-to-definition (including nested member access), find references, completion (`.` `:` `@` `?` triggers), signature help, document/workspace symbols, rename with prepare, folding ranges, inlay hints, call hierarchy, semantic tokens, formatting, code actions (quick fixes, refactorings, import management), code lens (Run/Build/Test).

---

## Well-Known Types

The compiler has special knowledge of these types (`src/well_known.rs`):

| Type | Special Handling |
|------|------------------|
| `Option<T>` | Pattern matching codegen, `?` operator |
| `Result<T,E>` | Pattern matching codegen, `?` operator |
| `Vec<T>` | Indexing, iteration |
| `String` | String interpolation, literals |
| `HashMap<K,V>` | Iteration |
| `Range` | Loop codegen |

---

## Extension Points

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

## Build & Test

```bash
cargo build --release          # Build compiler
cargo test --lib               # Run unit tests (146 tests)
cargo test --all               # Run all tests
./target/release/zen FILE      # Run a .zen file
./target/release/zen-lsp       # Start LSP
```

---

## Related Documentation

- `docs/INTRINSICS_REFERENCE.md` — Compiler intrinsics reference
- `docs/ROADMAP.md` — Development roadmap
- `docs/design/STDLIB_DESIGN.md` — Stdlib API design
- `docs/design/TYPE_SYSTEM_CLEANUP.md` — Type system cleanup plan
- `docs/design/SEPARATION_OF_CONCERNS.md` — Three-layer architecture
- `docs/QUICK_START.md` — Getting started guide
