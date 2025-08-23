# Lynlang Global Memory

## Project Overview
**Lynlang (Zen)** - A systems programming language with LLVM backend, written in Rust.
- Branch: `ragemode`
- Language: Rust
- Backend: LLVM
- Test Coverage: Excellent (214 tests, 100% passing)

## Key Architectural Components

### Core Systems
1. **Parser** (`src/parser/`) - Complete implementation with all language features
2. **Type System** (`src/type_system/`) - Generic type instantiation and monomorphization
3. **Type Checker** (`src/typechecker/`) - Behavior resolution and type checking
4. **LLVM Codegen** (`src/codegen/llvm/`) - 13 modules for IR generation

### Recent Major Achievements (August 2025)
1. **Generic Type System** - Complete foundation with TypeEnvironment, TypeSubstitution, TypeInstantiator, and Monomorphizer
2. **Behavior/Trait System** - Full implementation from AST to LLVM vtable generation
3. **Enhanced LLVM Infrastructure** - Vtable generation, method resolution, static/dynamic dispatch

## Critical Integration Points

### Type System ↔ LLVM Codegen
- **Gap**: Generic types parse correctly but LLVM codegen integration incomplete
- **Solution**: Bridge TypeInstantiator with LLVMCompiler::compile_generic_function

### Comptime ↔ Compilation Pipeline
- **Gap**: Evaluator exists but not hooked into main compilation flow
- **Solution**: Integrate ComptimeEvaluator in Compiler::compile before LLVM generation

### Behavior System ↔ Runtime
- **Status**: Foundation complete, vtables generating
- **Next**: Dynamic dispatch runtime support

## Code Quality Metrics
- **Warnings**: 40+ unused implementation warnings (mostly in LLVM codegen)
- **Debug Code**: Print statements in src/codegen/llvm/functions.rs need removal
- **Test Health**: 100% pass rate across 214 tests

## Key File Locations
- Entry Point: `src/main.rs`
- Compiler Driver: `src/compiler.rs`
- Test Suite: `tests/` (28 test files)
- LLVM Main: `src/codegen/llvm/compiler.rs`

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