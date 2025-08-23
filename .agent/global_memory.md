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
- Conditional expressions with pattern matching ✅ (fixed today)
- Basic struct support
- String literals
- Pointers (basic implementation)
- FFI declarations (partial)

## Next Priorities (from ROADMAP)
1. **C FFI Enhancement** - Complete external functions, C compatibility
2. **Comptime Evaluation Engine** - Build actual compile-time execution
3. **Enhanced Type System** - Arrays, enums, better structs
4. **Module System** - Imports, visibility, namespaces
5. **Loop Constructs** - Unified loop syntax implementation
6. **Behaviors/Traits** - Contract-based polymorphism
7. **Memory Management** - Allocators, smart references

## Recent Progress (Current Session)
- ✅ Added generic type parsing support (List<T>, Map<K,V>)
- ✅ Implemented generic function parsing (fn map<T, U>)
- ✅ Enhanced struct parsing for generic parameters
- ✅ Fixed parser to distinguish generic functions from structs
- ✅ Added comprehensive test suite for generics (7 tests)
- ✅ Fixed pointer type parsing for both Symbol and Operator tokens
- ✅ All tests passing (116 total, verified 2025-08-23)

## Known Issues
- Git push timeouts occasionally  
- Comptime parser exists but no evaluation engine yet
- Type system needs major expansion
- No real module system yet

## Critical Files
- `/src/codegen/llvm/` - LLVM IR generation
- `/src/parser/` - Language parser (COMPLETE)
- `/src/ast.rs` - AST definitions
- `/src/comptime.rs` - New comptime module (needs implementation)
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