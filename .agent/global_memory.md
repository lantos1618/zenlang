# Zen Language Global Memory

## Project State (as of 2025-08-27 - Latest Session - Updated)
- **Completion**: ~80% of compiler complete (major parser improvements)
- **Language**: Rust-based compiler targeting LLVM
- **Location**: /home/ubuntu/zenlang
- **Branch**: ragemode
- **Test Status**: 284/285 tests passing (99.6% pass rate - 1 vector test failing)
- **Recent Focus**: FFI test verification and stdlib development
- **FFI Tests**: All 7 tests now properly verify stdout output

## Key Features Working
- Basic types, functions, variables (mutable/immutable)
- Structs with field access
- Pattern matching with ? operator
- Basic generics with monomorphization
- C FFI (extern functions)
- Arrays (fixed-size)
- @std namespace with core/build/io/net modules
- Result<T,E> and Option<T> types
- String interpolation $(expr) fully working!
- Loops fully spec-compliant (condition, range, iterator)
- **NEW**: Comptime execution framework with interpreter
- **NEW**: Self-hosted lexer, parser, stdlib written in Zen
- **NEW**: Network module with TCP/UDP support

## Recent Improvements (2025-08-27)
1. ✅ **Parser Enhanced**: Generic struct literals, member assignments, array indexing chains
2. ✅ **Block Expressions**: Added for pattern matching arms
3. ✅ **Operator Fixes**: Proper && and || tokenization
4. ✅ **External Functions**: Optional parameter names in declarations
5. ✅ **Pattern Matching**: Boolean literals as patterns
6. ✅ **Test Coverage**: 9/10 stdlib vector tests now passing

## Previous Session (2025-08-26)
1. ✅ **Compilation Errors Fixed**: Resolved all comptime module conflicts
2. ✅ **Test Infrastructure**: Output capture working correctly
3. ✅ **Standard Library Expanded**: Math, collections, algorithms modules

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