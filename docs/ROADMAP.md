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
- [ ] macOS syscalls
- [ ] Windows syscalls

### 4. Remaining Type System Cleanup
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
