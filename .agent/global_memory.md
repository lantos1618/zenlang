# Zen Language - Global Memory

## Project Overview
- **Language**: Zen - A modern systems programming language
- **Current Implementation**: Rust-based compiler with LLVM backend
- **Goal**: Self-hosted compiler with comprehensive standard library

## Current Status (2025-08-27)
- Loop syntax already converted to functional approach ✓
- No changes needed to parser/lexer/codegen
- Standard library modules partially implemented
- Working towards self-hosting

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

## Standard Library Progress
### Existing Modules
- ✓ core.zen - Basic types and functions
- ✓ io.zen - Input/output operations  
- ✓ iterator.zen - Iteration utilities

### To Implement
- [ ] mem.zen - Memory management
- [ ] math.zen - Mathematical functions
- [ ] string.zen - String utilities
- [ ] collections.zen - Data structures
- [ ] fs.zen - File system operations
- [ ] net.zen - Network operations
- [ ] async.zen - Async/await utilities

## Design Principles
- No keywords philosophy - composable primitives
- Pattern matching with `?` operator
- Explicit error handling with Result<T,E>
- Compile-time metaprogramming
- Simplicity, elegance, practicality
- DRY & KISS principles

## Recent Commits
- d7746ce: Session summary documentation
- 9a0e96a: Updated loop syntax documentation to functional approach ✓
- 14b4a17: Added comprehensive self-hosted test suites
- 96e0c34: Added type casting and improved language features
- 5255e43: Fixed loop syntax parsing and improved test suite

## Next Steps
1. Clean up any remaining legacy loop references
2. Implement core standard library modules in Zen
3. Continue working towards self-hosting
4. Maintain high test coverage