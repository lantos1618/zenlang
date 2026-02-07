# Zen Language Roadmap

**Status**: Late Alpha | **Updated**: February 2026

---

## What Works

- Zero-keyword syntax with pattern matching (`?`)
- Type system: structs, enums, generics, Option<T>, Result<T,E>
- UFC (Uniform Function Call)
- Zig-style allocators (GPA, Arena, Pool)
- Collections: Vec, String, HashMap, Stack, Queue, Set, LinkedList
- Safe pointers: Ptr<T>, MutPtr<T>, RawPtr<T>
- Syscall-based I/O (Linux x86-64)
- Full LSP support (16K LOC — completion, hover, navigation, refactoring)
- 25+ compiler intrinsics
- Compile-time evaluation with AST introspection (comptime/meta)
- 143 passing tests

---

## Recent Improvements (Feb 2026)

- **TypeStore** — Single source of truth for all type information
- **name_utils** — Canonical key construction, unified method key format (`"Type.method"`)
- **StructInfo indexing** — O(1) field lookups via lazy HashMap
- **Comptime restructure** — Proper control flow enum, `with_scope()`, meta API extracted to 4 files
- **Fragile code audit** — 132/135 issues fixed across parser, typechecker, codegen, comptime
- **LSP cleanup** — strip_generics via name_utils, removed hardcoded StdModule members

---

## Current Priorities

### 1. Module System Fixes
Generic type resolution across module boundaries has issues:
- Monomorphization of imported generic types fails
- Type inference for behavior implementations incomplete

This blocks runtime testing of stdlib from .zen files.

### 2. Iterator Combinators
Need first-class closures for:
```zen
vec.iter().map(fn).filter(fn).collect()
```

Currently only specialized methods work (sum, product, min, max).

### 3. Cross-Platform Support

**Current state:** Syscalls are 100% manual — the compiler generates inline x86-64
assembly (`syscall` instruction with rax/rdi/rsi/rdx/r10/r8/r9 register convention).
LLVM provides zero platform abstraction for this. Every stdlib file using I/O,
networking, threading, or memory-mapping depends on Linux x86-64 syscall numbers
hardcoded in `stdlib/sys/syscall.zen`.

**What needs to change per platform:**

| Platform | Syscall Numbers | Registers | Instruction | Effort |
|----------|----------------|-----------|-------------|--------|
| Linux ARM64 | Different | x0-x7 | `svc #0` | Medium |
| macOS x86-64 | Different (exit=1 not 60) | Same | `syscall` | Medium |
| macOS ARM64 | Different | x0-x7 | `svc #0x80` | Medium-high |
| Windows | N/A — uses NTAPI | Completely different | No `syscall` | Very high |

**Phased approach:**

- [ ] **Phase 1: Target detection** — Query LLVM target triple, route to platform-specific codegen in `build_syscall()`
- [ ] **Phase 2: Linux ARM64** — Easiest win. Same syscall concept, different ABI. Create `syscall_aarch64.zen` with ARM64 numbers, modify `build_syscall()` for `svc #0` + x0-x7 registers
- [ ] **Phase 3: macOS** — Create `syscall_macos.zen`. Note: Apple discourages raw syscalls (numbers change between versions), long-term should FFI to libSystem
- [ ] **Phase 4: Windows** — Architectural change. No raw syscalls — needs NTAPI via FFI (`ntdll.dll`/`kernel32.dll`). Essentially a separate I/O backend
- [ ] **Phase 5: Stdlib abstraction layer** — Platform-independent I/O/threading/memory API in stdlib that dispatches to OS-specific implementations

**Files affected:** `src/codegen/llvm/stdlib_codegen/compiler.rs` (build_syscall), `src/intrinsics.rs`, `stdlib/sys/syscall.zen`, and every stdlib file using `compiler.syscall*()` (~15 files across io/, sys/, concurrency/, memory/)

### 4. LSP Consolidation

The LSP is 16,399 LOC across 55 files — reasonable for a full-featured LSP with
compiler integration (rust-analyzer is 150K, gopls 50K), but has ~1,500 LOC of
consolidation opportunities:

- [ ] **Consolidate type inference** — 8 separate files duplicate Expression-matching logic (~500 LOC savings)
- [ ] **Merge hover submodules** — 9 files → 4 files (~250 LOC savings)
- [ ] **Merge document_store submodules** — 9 files → 4 files (~250 LOC savings)
- [ ] **Error handling macros** — Replace 50+ verbose lock/parse patterns (~150 LOC savings)

### 5. Remaining Type System Cleanup
- Replace remaining string-based type checks (~10 locations)
- Create TypeAliasRegistry for StaticString/String normalization
- Implement generic type substitution in stdlib method resolution
- Remove hardcoded method type inference once stdlib resolution verified

---

## Self-Hosting Path

1. Intrinsics only in Rust compiler (mostly done)
2. Stdlib in Zen (done)
3. Lexer/Parser in Zen
4. Typechecker in Zen
5. Codegen in Zen (LLVM bindings or custom backend)

---

## Commands

```bash
cargo build --release          # Build compiler
cargo test --all               # Run tests
./target/release/zen FILE      # Run .zen file
./target/release/zen-lsp       # Start LSP
```
