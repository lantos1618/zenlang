# Lynlang Development Plan

## Current Sprint: Type System & LLVM Integration
**Goal**: Complete the bridge between advanced type features and code generation

### Phase 1: Code Quality & Cleanup (Today)
1. ✅ Set up .agent meta files
2. ⏳ Remove debug print statements from functions.rs
3. ⏳ Address unused code warnings in LLVM modules
4. ⏳ Verify all tests still pass

### Phase 2: Generic Type Integration (Priority 1)
1. Connect TypeInstantiator to LLVM compiler
2. Implement generic function compilation in LLVM
3. Add generic struct compilation support
4. Create integration tests for generic code generation
5. Ensure monomorphization works end-to-end

### Phase 3: Comptime Pipeline (Priority 2)
1. Hook ComptimeEvaluator into Compiler::compile
2. Process comptime blocks before LLVM generation
3. Cache comptime evaluation results
4. Add comptime tests to ensure correctness

### Phase 4: Enhanced Type Features (Priority 3)
1. Implement array types with size `[T; N]`
2. Improve enum variant handling in codegen
3. Add type alias support
4. Create comprehensive type system tests

## Future Milestones

### Milestone 1: Module System
- Design import/export syntax
- Implement module resolution
- Add visibility modifiers
- Create module dependency graph

### Milestone 2: Standard Library Foundation
- Core collections (Vec, HashMap, String)
- Basic I/O abstractions
- Memory allocator interface
- Common traits/behaviors

### Milestone 3: Advanced Features
- Async/await support
- Closure implementation
- Pattern matching in codegen
- Macro system design

## Testing Strategy
- Maintain 100% test pass rate
- Add integration tests for each new feature
- Use property-based testing for type system
- Create end-to-end compilation tests

## Git Workflow
- Commit after each completed feature
- Push to GitHub every 2-3 commits
- Create issues for bugs found during development
- Use descriptive commit messages

## Success Metrics
- Zero failing tests
- No compiler warnings
- Clean code with no debug statements
- All features have corresponding tests
- Documentation for public APIs