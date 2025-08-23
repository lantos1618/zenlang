# Lynlang Implementation Plan
**Updated**: 2025-08-23 (Current Session)
**Branch**: ragemode

## Mission
Complete the Lynlang compiler to production readiness with focus on core language features, type system, and runtime capabilities.

## Project Zen Principles
1. **Simplicity**: Clear, readable code over clever tricks
2. **Elegance**: Well-structured systems that compose naturally  
3. **Practicality**: Focus on working features, not perfection
4. **Intelligence**: Make smart trade-offs, prioritize impact

## Current Status ✅
- **164 tests passing** - All green!
- **Pattern matching**: ✅ COMPLETE (parser and codegen working)
- **Comptime**: Parser done, evaluator needs integration
- **C FFI**: Basic support implemented
- **Array literals**: ✅ COMPLETE

## Sprint Focus (Priority Order)

### 1. 🎯 Generic Type Instantiation
**Priority**: HIGHEST - Foundation for advanced features
**Status**: Not started
- [ ] Design generic type parameter syntax in AST
- [ ] Implement type parameter parsing
- [ ] Build type instantiation engine
- [ ] Add monomorphization in codegen
- [ ] Create comprehensive test suite

### 2. 🎯 Trait/Behavior System
**Priority**: HIGH - Enables polymorphism
**Status**: Not started
- [ ] Define trait/behavior syntax
- [ ] Implement trait parsing
- [ ] Build trait resolution system
- [ ] Add trait bounds checking
- [ ] Implement trait objects

### 3. 🎯 Comptime Evaluation Engine
**Priority**: HIGH - Already partially implemented
**Status**: Parser done, evaluator exists but not integrated
- [ ] Hook evaluator into compilation pipeline
- [ ] Implement comptime function execution
- [ ] Add comptime type generation
- [ ] Enable compile-time assertions
- [ ] Test with real examples

### 4. Enhanced Type System
**Status**: Basic types working
- [ ] Array types with size (`[T; N]`)
- [ ] Better enum variant handling
- [ ] Type aliases (`type Name = ...`)
- [ ] Option/Result improvements

### 5. Standard Library
**Status**: Not started
- [ ] Core types (Vec, HashMap, String)
- [ ] I/O operations
- [ ] Memory management utilities
- [ ] Collections and iterators

### 6. Module System
**Status**: Not started
- [ ] Import/export syntax
- [ ] Module resolution
- [ ] Visibility rules
- [ ] Package management

## Development Strategy
- **80% implementation, 20% testing** ratio
- Write tests before implementation when possible
- Small, incremental changes
- Clean up as we go
- Maintain 100% test pass rate

## Success Metrics
- All tests passing continuously
- Generic collections working (Vec<T>, HashMap<K,V>)
- Trait-based polymorphism functional
- Comptime reduces runtime overhead
- Can build real-world applications

## Daily Workflow
1. Review test status
2. Pick highest priority task
3. Write tests first
4. Implement until tests pass
5. Refactor and clean up
6. Update docs and meta files
7. Commit with clear message

## Session Management
- Maintain context at ~40% (100-140k tokens)
- Use .agent directory for state
- Clean up temporary files
- Update global_memory.md regularly
- Track progress in todos.md