# Zen Language Global Memory

## Project Overview
- **Language**: Zen - A modern systems programming language
- **Philosophy**: No keywords, pattern matching everything, explicit error handling
- **Current Status**: 50-55% complete
- **Goal**: Self-hosted compiler by 2026
- **Lines of Code**: ~5,450 Rust LOC across 29 files
- **Test Coverage**: 36 test files, 100% pass rate

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

## Critical Missing Features
1. String interpolation $(expr)
2. Complete enum codegen
3. Module import system
4. Spec-compliant loops
5. Break/continue statements
6. Advanced generics
7. Collections library
8. Memory management system
9. Behaviors/traits
10. Comptime evaluation

## Testing Issue to Fix
- External function calls (printf/puts) generate correct LLVM IR but aren't actually executed/verified in tests
- Need to add integration tests that capture and verify stdout output

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