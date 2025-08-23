# Lynlang Active TODOs

## In Progress
- [ ] Set up .agent meta information files

## Today's Focus
- [ ] Clean up debug statements in src/codegen/llvm/functions.rs
- [ ] Address unused code warnings in LLVM modules
- [ ] Integrate generic types with LLVM codegen
- [ ] Hook comptime evaluator into compilation pipeline
- [ ] Run comprehensive test suite
- [ ] Commit and push progress

## This Week
- [ ] Complete generic type LLVM integration
- [ ] Implement generic function compilation
- [ ] Add generic struct compilation
- [ ] Create integration tests for generics
- [ ] Ensure monomorphization works end-to-end

## Backlog (Priority Order)

### High Priority
- [ ] Comptime pipeline integration
- [ ] Array types with size [T; N]
- [ ] Improved enum variant handling
- [ ] Type alias support

### Medium Priority
- [ ] Basic module system design
- [ ] Import/export implementation
- [ ] Standard library Vec implementation
- [ ] Standard library HashMap implementation

### Low Priority
- [ ] Async/await design
- [ ] Closure implementation
- [ ] Macro system design
- [ ] Property-based testing framework

## Completed Recently
- [x] Generic type system foundation
- [x] Behavior/trait system foundation
- [x] LLVM vtable generation
- [x] Parser for all language features
- [x] Basic pattern matching support

## Known Issues
- Debug print statements in production code
- 40+ unused implementation warnings
- Generic types not connected to LLVM
- Comptime evaluator not in pipeline
- No module system yet

## Notes
- All 214 tests currently passing
- Maintain test coverage with new features
- Use frequent git commits for progress tracking
- Clean up after completing each phase