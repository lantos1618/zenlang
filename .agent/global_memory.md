# Zen Language Project - Global Memory

## Project Overview
**Zen** is a modern systems programming language that aims to replace traditional languages like C/C++/Rust with a more elegant, composable syntax. The project consists of a compiler written in Rust that targets LLVM IR.

## Current Implementation Status (August 2025)

### What Works (Can compile and run) ✅
- **Basic Compiler**: 45-50% complete, ~10,500 lines of Rust code across 50 files
- **Core Language Features**:
  - Function declarations with `=` syntax: `add = (a: i32, b: i32) i32 { a + b }`
  - Variable declarations: `:=` (immutable), `::=` (mutable)
  - Basic types: i32, i64, f32, f64, bool
  - Arithmetic and comparison operators
  - Function calls and returns
  - LLVM code generation working
- **Test Suite**: 35 test suites, 100% pass rate
- **Standard Library Foundation**: @std namespace, Result<T,E>, Option<T>, basic IO module

### Partially Working 🚧
- **Structs**: Parser complete, codegen incomplete
- **Pattern Matching**: Parser complete, codegen work in progress  
- **Loops**: Basic version works, spec-compliant version WIP
- **Type Checking**: Basic implementation exists
- **Generics**: Parsing works, instantiation/monomorphization incomplete

### Major Missing Features ❌
- **String interpolation**: `$(expr)` syntax not implemented
- **Full comptime system**: Partial implementation only
- **Behaviors**: Trait/interface system not implemented
- **UFCS**: Uniform Function Call Syntax missing
- **Memory management**: Ptr<T>, Ref<T>, allocators not implemented
- **Async/await**: Not implemented
- **Module system**: Import system incomplete
- **Complete standard library**: Only basic modules exist

## Architecture Overview

### Compiler Structure (src/)
```
parser/      - 12 files, parsing all language constructs
codegen/     - LLVM IR generation (15 files)
typechecker/ - Type system and inference (5 files)
stdlib/      - @std namespace implementation (4 files)
ast.rs       - Abstract Syntax Tree definitions
compiler.rs  - Main compiler driver
lexer.rs     - Tokenization
```

### Key Design Principles
1. **Minimal Keywords**: Uses `?` for all conditionals, `=` for functions, `:=` family for variables
2. **Explicit Error Handling**: No exceptions, Result<T,E> and Option<T> types
3. **Compile-time Everything**: Heavy use of `comptime` for metaprogramming
4. **@std Bootstrap**: Special `@std.core` and `@std.build` provide compiler intrinsics

## Self-Hosting Requirements Analysis

### What's Needed for Basic Self-Hosting
A self-hosted Zen compiler needs to compile itself. Current Rust implementation is ~10,500 lines, so Zen version would need similar functionality:

1. **Complete Language Implementation**: All syntax from lang.md must work
2. **Module System**: Must be able to import/organize large codebases
3. **Standard Library**: Collections, memory management, IO, OS interface
4. **LLVM Integration**: Either FFI to LLVM or custom backend
5. **Build System**: Equivalent to Cargo for managing projects

### Critical Path Analysis
The path to self-hosting involves three major phases:

**Phase 1 - Language Completion** (foundational features)
- Complete structs, enums, pattern matching codegen
- Full generic instantiation and monomorphization
- String interpolation
- Module import system
- Basic collections (Array, Map)

**Phase 2 - Standard Library** (essential for real programs)
- Memory management and allocators  
- File/IO operations
- String manipulation
- Error handling utilities
- System interface (process, environment)

**Phase 3 - Advanced Features** (needed for compiler complexity)
- Complete comptime system
- Behaviors (trait system)
- UFCS for clean APIs
- Async/await for build tools

## Development History
- Started as systems language experiment
- Evolved through multiple syntax iterations
- Recently achieved 100% test pass rate
- Added @std namespace and Result/Option types
- Now approaching feature completeness for basic programs

## Team/Maintainer Context
- Single maintainer project currently
- Uses test-driven development approach
- All features tested before merging
- Follows incremental implementation strategy
- Focus on correctness over speed of development
