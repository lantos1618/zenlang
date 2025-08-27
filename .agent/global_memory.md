# Zen Language - Global Memory

## Project Overview
- **Language**: Zen - A modern systems programming language
- **Current Implementation**: Rust-based compiler with LLVM backend
- **Goal**: Self-hosted compiler with comprehensive standard library

## Current Status (2025-08-27)
- Loop syntax fully converted to functional approach ✅
- Documentation cleaned up - no old syntax references ✅
- Standard library expanded with critical modules ✅
- Self-hosted parser 100% complete ✅
- Test suite: 228/234 tests passing (97.4% success rate)

## Key Files & Locations
### Core Implementation
- Parser: `/src/parser/statements.rs` (loop parsing at lines 306-508)
- AST: `/src/ast.rs` (loop definitions at lines 277-283)
- Lexer: `/src/lexer.rs` (keywords at lines 5-6, 345-346)
- Codegen: `/src/codegen/llvm/statements.rs` (loop codegen at lines 289-340)

### Standard Library
- Core: `/stdlib/core.zen` (Range implementation at lines 66-94)
- Iterator: `/stdlib/iterator.zen` (iteration methods)
- IO: `/stdlib/io.zen` (input/output operations)

### Documentation
- Language Reference: `/.agent/zen_language_reference.md`
- Language Guide: `/lang.md`
- User Guide: `/ZEN_GUIDE.md`

## Loop Syntax Status
### Current Working Syntax
```zen
// Functional range iteration
range(0, 10).loop(i -> { })
range_inclusive(1, 5).loop(i -> { })

// Simple loops
loop condition { }  // while-like
loop { }           // infinite
```

### Removed/Legacy Syntax
```zen
// NO LONGER SUPPORTED:
// loop i in 0..10 { }
// loop item in items { }
// These have been replaced with functional approach
```

## Standard Library Progress (24 modules total)
### Core Modules (Complete)
- ✓ core.zen - Basic types and functions
- ✓ io.zen - Input/output operations  
- ✓ iterator.zen - Iteration utilities
- ✓ mem.zen - Memory management
- ✓ math.zen - Mathematical functions
- ✓ string.zen - String utilities
- ✓ collections.zen - Data structures
- ✓ fs.zen - File system operations
- ✓ net.zen - Network operations
- ✓ vec.zen - Dynamic arrays
- ✓ hashmap.zen - Hash map implementation
- ✓ algorithms.zen - Common algorithms

### New Modules Added (2025-08-27)
- ✓ assert.zen - Testing and assertion utilities
- ✓ process.zen - Process management
- ✓ thread.zen - Threading and concurrency

### Compiler Support Modules
- ✓ lexer.zen - Tokenization (90% complete)
- ✅ parser.zen - Parsing (100% complete)

### To Implement for Full Self-Hosting
- [ ] ast.zen - Abstract syntax tree
- [ ] type_checker.zen - Type checking
- [ ] codegen.zen - Code generation
- [ ] async.zen - Async/await utilities
- [ ] test_framework.zen - Testing infrastructure

## Design Principles
- No keywords philosophy - composable primitives
- Pattern matching with `?` operator
- Explicit error handling with Result<T,E>
- Compile-time metaprogramming
- Simplicity, elegance, practicality
- DRY & KISS principles

## Recent Commits (2025-08-27)
- 35863ae: Complete loop syntax migration and self-hosted parser
- 3709c52: Added critical stdlib modules (assert, process, thread)
- 7f196bf: Cleaned up references to old loop syntax
- cbf3787: Refactored loop syntax to use parentheses
- 078714e: Removed unused 'In' keyword from lexer
- d7746ce: Session summary documentation
- 9a0e96a: Updated loop syntax documentation to functional approach ✓
- 14b4a17: Added comprehensive self-hosted test suites

## Next Steps
1. Fix 6 failing test cases (function pointers, array ops, multiple returns) - existing compiler issues
2. ✅ Implement ast.zen module for self-hosting - COMPLETED
3. ✅ Implement type_checker.zen module - COMPLETED
4. ✅ Implement codegen.zen module - COMPLETED
5. Achieve 100% test pass rate - 228/234 tests passing (97.4%)
6. Ready to merge to main branch