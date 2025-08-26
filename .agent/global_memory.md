# Zen Language Global Memory

## Project State (as of 2025-08-26 - Latest Session)
- **Completion**: ~65% of compiler complete
- **Language**: Rust-based compiler targeting LLVM
- **Location**: /home/ubuntu/zenlang
- **Branch**: ragemode
- **Test Status**: 285 tests passing (100% pass rate)
- **Recent Focus**: Completing core language features for self-hosting

## Key Features Working
- Basic types, functions, variables (mutable/immutable)
- Structs with field access
- Pattern matching with ? operator
- Basic generics with monomorphization
- C FFI (extern functions)
- Arrays (fixed-size)
- @std namespace with core/build/io modules
- Result<T,E> and Option<T> types
- **NEW**: String interpolation $(expr) fully working!
- **NEW**: Loops fully spec-compliant (condition, range, iterator)

## Critical Issues RESOLVED
1. ✅ **Printf/puts tests**: test_output_verification.rs properly captures output
2. ✅ **String interpolation**: Fixed string variable handling, now working correctly
3. ✅ **Loop syntax**: Already spec-compliant with all loop types

## Architecture
- Lexer -> Parser -> AST -> Type Checker -> LLVM Codegen
- Standard library modules in src/stdlib/
- Tests in tests/ (38 test files, mostly passing)
- Examples in examples/ (*_working.zen files work)

## Development Principles
- 80% implementation, 20% testing 
- DRY & KISS
- Frequent git commits
- Work best at 40% context (100K-140K tokens)
- Use .agent/ folder for metadata
- Cleanup after tasks complete

## Next Major Milestone
Self-hosted compiler with comprehensive standard library written in Zen