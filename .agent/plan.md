# Lynlang Development Plan

## Current Focus: Pattern Matching & Comptime

### Phase 1: Complete Pattern Matching (TODAY)
- [ ] Fix pattern matching codegen in LLVM
- [ ] Ensure all pattern matching tests pass
- [ ] Handle edge cases in match expressions

### Phase 2: Comptime Evaluation (HIGH PRIORITY)
- [ ] Build evaluation engine for compile-time expressions
- [ ] Implement comptime function evaluation
- [ ] Add comptime type-level programming support

### Phase 3: Core Features
- [ ] Generic type system improvements
- [ ] Trait/behavior system implementation
- [ ] Type checker enhancements

## Testing Strategy
- Run tests after each significant change
- Maintain 100% test pass rate
- Add new tests for new features

## Workflow
1. Make focused changes
2. Test immediately
3. Commit frequently with clear messages
4. Push to remote regularly