# Zen Language Global Memory

## Project State (as of 2025-08-26 - Latest Session)
- **Completion**: ~75% of compiler complete
- **Language**: Rust-based compiler targeting LLVM
- **Location**: /home/ubuntu/zenlang
- **Branch**: ragemode
- **Test Status**: 281/285 tests passing (98.5% pass rate - 4 parsing tests failing)
- **Recent Focus**: Self-hosting components and comptime framework

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

## Recent Improvements (2025-08-26)
1. ✅ **Compilation Errors Fixed**: Resolved all comptime module conflicts and type mismatches
2. ✅ **Test Infrastructure**: Output capture working correctly for printf/puts
3. ✅ **Standard Library Expanded**: Added comprehensive stdlib modules in Zen:
   - math.zen: Mathematical functions and constants
   - collections.zen: Stack, Queue, Deque, LinkedList, BST, Set
   - algorithms.zen: Sorting, searching, and functional operations

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