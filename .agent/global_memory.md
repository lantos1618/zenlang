# Zen Language Global Memory

## Project Overview
- **Language**: Zen - A modern systems programming language
- **Philosophy**: No keywords, pattern matching everything, explicit error handling
- **Current Status**: 55-60% complete
- **Goal**: Self-hosted compiler by 2026
- **Lines of Code**: ~5,450 Rust LOC across 29 files
- **Test Coverage**: 42 test suites, 100% pass rate (all 273 tests passing)

## Architecture
- **Components**: Lexer → Parser → AST → Type Checker → LLVM Codegen
- **Backend**: LLVM for native code generation
- **Testing**: Comprehensive unit and integration tests

## Key Features Working
- Functions, variables, basic types
- Pattern matching with ? operator
- Structs with field access
- Arrays (fixed-size)
- Pointer operations
- @std namespace foundation
- Result<T,E> and Option<T> types
- IO module with file operations
- String interpolation $(expr) ✅ FIXED
- FFI output tests (printf/puts) ✅ WORKING

## Critical Missing Features
1. Complete enum codegen
2. Module import system  
3. Spec-compliant loops (for-each style)
4. Break/continue statements
5. Advanced generics instantiation
6. Collections library
7. Memory management system
8. Behaviors/traits
9. Comptime evaluation
10. Pattern matching codegen (parser done)

## Self-Hosting Requirements
- Port 5,450 lines of Rust compiler to Zen
- LLVM FFI bindings
- Complete standard library
- Bootstrap process

## Code Principles
- DRY (Don't Repeat Yourself)
- KISS (Keep It Simple, Stupid)
- Test-driven development
- Frequent git commits
- Clean up after completion