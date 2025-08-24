# Lynlang Implementation Plan

## Project Status
- **Core Language**: Working (functions, variables, control flow, FFI)
- **Parser**: Complete (all syntax features implemented)
- **Tests**: 165/165 passing (100% pass rate)
- **LLVM Backend**: Functional for basic features

## Phase 1: Foundation (Current Focus)
### 1.1 Comptime Evaluation Engine ⏳
- Parser support: ✅ Complete
- Evaluation engine: ❌ TODO
- Implementation approach:
  1. Create `comptime_eval.rs` module
  2. Build AST interpreter for comptime blocks
  3. Support constant folding and compile-time function execution
  4. Integrate with type checker for dependent types

### 1.2 Dedicated Type Checker 🔧
- Current state: Mixed with codegen
- Goal: Separate type checking phase
- Steps:
  1. Create `src/type_checker.rs`
  2. Extract type checking logic from codegen
  3. Build symbol table and type environment
  4. Implement type inference engine
  5. Add better error messages with suggestions

## Phase 2: Advanced Type System
### 2.1 Generic Types & Monomorphization
- Type parameters parsing: ✅ Done
- Type instantiation: ❌ TODO
- Monomorphization: ❌ TODO
- Implementation:
  1. Track generic type parameters
  2. Build type substitution system
  3. Generate specialized versions during codegen
  4. Cache monomorphized functions

### 2.2 Trait/Behavior System
- Design decisions needed:
  - Rust-like traits vs Go-like interfaces
  - Explicit impl blocks vs structural typing
- Implementation plan:
  1. Define trait syntax and semantics
  2. Add trait resolution
  3. Support trait bounds on generics
  4. Implement trait objects for dynamic dispatch

## Phase 3: Memory & Runtime
### 3.1 Memory Management
- Stack allocation: ✅ Working
- Heap allocation: ❌ TODO
- Allocator interface: ❌ TODO
- Reference counting/borrowing: ❌ TODO

### 3.2 Module System
- File-based modules: ❌ TODO
- Namespace management: ❌ TODO
- Visibility rules: ❌ TODO
- Package management: ❌ TODO

## Phase 4: Standard Library
### 4.1 Core Types
- Vec<T>: Dynamic arrays
- HashMap<K,V>: Hash tables
- String: UTF-8 strings
- Option<T> & Result<T,E>: Already parsed

### 4.2 Core Modules
- io: File and console I/O
- mem: Memory utilities
- thread: Concurrency primitives
- net: Networking

## Phase 5: Advanced Features
### 5.1 Async/Await
- Task<T> type
- Async runtime
- Async function syntax
- await operator

### 5.2 Tooling
- Language server improvements
- Debugger support
- Code formatter
- Documentation generator

## Implementation Strategy
1. **Maintain 100% test coverage** - Never let tests fail
2. **Incremental development** - Small, working changes
3. **Clean architecture** - Separate concerns properly
4. **User feedback** - Use GitHub issues for tracking
5. **Regular commits** - git commit after each feature

## Current Sprint (Next 48 hours)
1. [ ] Implement comptime evaluation engine
2. [ ] Extract type checker from codegen
3. [ ] Add basic generic type support
4. [ ] Start trait system design
5. [ ] Clean up unused code warnings

## Success Metrics
- Test pass rate: Maintain 100%
- Compilation speed: < 1s for 1000 LOC
- Error quality: Clear, actionable messages
- Code quality: No warnings, clean architecture