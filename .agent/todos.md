# Lynlang Active TODOs

## Completed Today (2025-08-23)
- [x] Integrated generic types with LLVM codegen (2/3 tests passing)
- [x] Implemented monomorphization with type inference
- [x] Added two-pass compilation (declare then define)
- [x] Cleaned up debug statements
- [x] Addressed critical unused code warnings

## In Progress
- [ ] Fix struct literal parsing for generic structs

## Today's Focus
- [ ] Fix remaining generic struct test
- [ ] Hook comptime evaluator into compilation pipeline
- [ ] Run comprehensive test suite
- [ ] Commit and push progress

## This Week
- [x] Generic type LLVM integration (mostly complete)
- [x] Generic function compilation working
- [ ] Generic struct compilation (parsing issue)
- [x] Integration tests for generics (2/3 passing)
- [x] Monomorphization works end-to-end

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
- [x] Generic function monomorphization
- [x] Type inference for generic instantiation
- [x] Behavior/trait system foundation
- [x] LLVM vtable generation
- [x] Parser for all language features
- [x] Basic pattern matching support

## Known Issues
- Struct literal parsing without explicit type parameters
- 90+ unused implementation warnings (acceptable for WIP)
- Comptime evaluator not fully integrated
- No module system yet

## Notes
- 2/3 generic LLVM tests passing
- Monomorphization pipeline complete
- Test coverage maintained
- Using frequent git commits for progress tracking