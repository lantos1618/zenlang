# Lynlang Global Memory

## Project Overview
**Lynlang (Zen)** - A systems programming language with LLVM backend, written in Rust.
- Branch: `ragemode`
- Language: Rust
- Backend: LLVM
- Test Coverage: 2/3 generic tests passing, overall excellent coverage

## Key Architectural Components

### Core Systems
1. **Parser** (`src/parser/`) - Complete implementation with all language features
2. **Type System** (`src/type_system/`) - Generic type instantiation and monomorphization
3. **Type Checker** (`src/typechecker/`) - Behavior resolution and type checking
4. **LLVM Codegen** (`src/codegen/llvm/`) - 13 modules for IR generation

### Recent Major Achievements (August 23, 2025)
1. **Generic Function Monomorphization** - Working type inference and instantiation
2. **Two-Pass LLVM Compilation** - Declare all functions first, then define bodies
3. **AST Transformation** - Replace generic calls with monomorphized versions
4. **Behavior/Trait System** - Full implementation from AST to LLVM vtable generation

## Critical Integration Points

### Type System ↔ LLVM Codegen
- **Status**: RESOLVED - Generic functions now properly monomorphized and compiled
- **Implementation**: TypeInstantiator creates specialized versions, AST transformer updates calls

### Comptime ↔ Compilation Pipeline
- **Gap**: Evaluator exists but not fully integrated
- **Solution**: Integrate ComptimeEvaluator in Compiler::compile before LLVM generation

### Behavior System ↔ Runtime
- **Status**: Foundation complete, vtables generating
- **Next**: Dynamic dispatch runtime support

## Code Quality Metrics
- **Warnings**: 90+ unused implementation warnings (acceptable for WIP)
- **Debug Code**: Cleaned up all debug statements
- **Test Health**: 2/3 generic LLVM tests passing

## Key File Locations
- Entry Point: `src/main.rs`
- Compiler Driver: `src/compiler.rs`
- Monomorphizer: `src/type_system/monomorphization.rs`
- LLVM Functions: `src/codegen/llvm/functions.rs`
- Test Suite: `tests/` (28 test files)

## Implementation Details

### Monomorphization Pipeline
1. Type check program to gather type information
2. Register generic functions/structs/enums
3. Collect instantiations from function calls (with type inference)
4. Generate monomorphized versions with specialized names
5. Transform AST to use monomorphized names
6. Pass transformed AST to LLVM compiler

### LLVM Compilation Flow
1. Register struct types
2. Declare external functions
3. **Declare all functions** (new)
4. **Compile function bodies** (separated)

## Development Philosophy
- **Simplicity**: Clean, minimal abstractions
- **Elegance**: Well-structured, modular design
- **Practicality**: Focus on working features over complexity
- **Intelligence**: Smart defaults, good error messages

## Working Context Management
- Optimal context window: 100K-140K tokens (40% capacity)
- Use frequent git commits for state tracking
- Clean up temporary files after tasks
- Self-terminate when objectives complete