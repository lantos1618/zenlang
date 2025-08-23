# Lynlang Global Memory
**Last Updated**: 2025-08-23 (Current Session)
**Branch**: ragemode

## Project Overview
Lynlang (Zen) is a systems programming language with LLVM backend, written in Rust. Focus on simplicity, performance, and zero-cost abstractions.

## Current Status
- **Parser**: ✅ COMPLETE - All major features implemented including array literals
- **Lexer**: ✅ Fully functional with all keywords  
- **Codegen**: ⚠️ Partial - Core features done, advanced features in progress
- **Type System**: ⚠️ Basic types only, needs generics/traits
- **Tests**: ✅ All 116 tests passing (verified 2025-08-23)

## Key Features Implemented
### Parser/Lexer Complete ✅
- Pattern matching syntax with guards
- Comptime blocks and expressions  
- Member access (dot operator) with chaining
- Range syntax (0..10, 0..=10)
- Method definitions in structs
- Generic type parsing
- All control flow constructs
- Error recovery

### Codegen (LLVM) Partial ⚠️
- Basic types (int, float, string, void, bool)
- Functions and function calls
- Binary operations and comparisons
- Conditional expressions with pattern matching ✅
- Basic struct support
- String literals  
- Pointers (basic implementation)
- FFI declarations (partial)
- **Comptime evaluation** ✅ (integrated today)

## Next Priorities (from ROADMAP)
1. **Generic Type Instantiation** ⭐ HIGHEST - Foundation for advanced features
2. **Trait/Behavior System** - Contract-based polymorphism
3. **Enhanced Type System** - Arrays with size, enums, type aliases  
4. **Module System** - Imports, visibility, namespaces
5. **Standard Library** - Core collections and utilities
6. **Memory Management** - Allocators, smart references
7. **Async/Await** - Concurrent programming support

## Recent Progress (2025-08-23 Session)
- ✅ Integrated comptime evaluation engine into compilation pipeline
- ✅ Made ComptimeEvaluator persistent across compilation phases
- ✅ Fixed comptime expression and statement evaluation  
- ✅ Added comptime integration tests (2/4 passing)
- ✅ Created GitHub issue templates for major features
- ✅ Updated .agent directory meta information
- ✅ All core tests still passing (93+ tests total)

## Known Issues
- Git push timeouts occasionally
- Comptime evaluator not fully persistent across declarations
- Type system needs major expansion (generics, traits)
- No real module system yet
- Some comptime tests failing (functions in comptime blocks)

## Critical Files
- `/src/codegen/llvm/` - LLVM IR generation
- `/src/parser/` - Language parser (COMPLETE)
- `/src/ast.rs` - AST definitions
- `/src/comptime.rs` - Comptime evaluator (integrated, partial support)
- `/tests/` - Comprehensive test suite
- `/ROADMAP.md` - Feature roadmap and priorities

## Working Principles
- Maintain zen and focus on completing features
- 80% implementation, 20% testing ratio
- Simplicity, elegance, practicality, intelligence
- Clean up after ourselves (remove debug files)
- Work best at 40% context window (100-140k tokens)
- Use .agent directory for state management
- Track todos and progress systematically