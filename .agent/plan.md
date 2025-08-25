# Zen Language Implementation Plan

## Current Status (2025-08-25)
✅ All 224 tests passing across 36 suites
✅ Language renamed from lynlang to zen
✅ Documentation and examples updated
✅ Project structure organized
✅ Parser supports both `::` and `=` for functions

## Critical Alignment Tasks with lang.md Spec

### Phase 1: Syntax Alignment (High Priority - Week 1)

#### 1.1 Parser Updates (2-3 days)
- [ ] Enforce `=` for function definitions only (deprecate `::`)
- [ ] Implement `::=` operator for mutable variable declarations
- [ ] Ensure `:=` for immutable bindings
- [ ] Remove if/else keywords from parser completely
- [ ] Implement `->` operator for pattern destructuring/binding
- [ ] Update all test files to use correct syntax

#### 1.2 Pattern Matching Enhancement (1-2 days)
- [ ] Ensure `?` operator is the ONLY conditional construct
- [ ] Implement guard clauses with `-> condition`
- [ ] Support struct destructuring with `->`
- [ ] Remove any if/else codegen paths
- [ ] Add comprehensive pattern matching tests

#### 1.3 Variable System (1 day)
- [ ] Implement `::=` for mutable variables (replace `mut` keyword)
- [ ] Ensure `:=` creates immutable bindings
- [ ] Support `: Type` (immutable) and `:: Type` (mutable) annotations
- [ ] Default initialization for declared but uninitialized variables

### Phase 2: Core Features (Week 2)

#### 2.1 Module System & @std Namespace (3-4 days)
- [ ] Implement @std.core with compiler intrinsics
- [ ] Implement @std.build for module imports
- [ ] Create build.import() mechanism
- [ ] Bootstrap core types (sizeOf, null_ptr, etc.)
- [ ] Design module resolution strategy

#### 2.2 Comptime Engine (4-5 days)
- [ ] Complete comptime evaluation engine (parser done)
- [ ] Support comptime blocks that evaluate to values
- [ ] Implement comptime parameters for generics
- [ ] Enable compile-time computations (tables, constants)
- [ ] Add comptime type introspection

### Phase 3: Type System (Week 3)

#### 3.1 Type Checker Separation (2-3 days)
- [ ] Extract type checking from codegen
- [ ] Create dedicated src/typechecker module
- [ ] Implement proper type inference
- [ ] Add rich type error reporting
- [ ] Build symbol table management

#### 3.2 Generic System (2-3 days)
- [ ] Generic type instantiation
- [ ] Monomorphization pass
- [ ] Type parameter constraints
- [ ] Generic type inference

#### 3.3 Behaviors/Traits (3-4 days)
- [ ] Design behavior syntax per lang.md
- [ ] Implement behavior definitions
- [ ] Add impl blocks for types
- [ ] Support static/dynamic dispatch
- [ ] Behavior-based polymorphism

### Phase 4: Standard Library (Week 4)

#### 4.1 Core Types (2-3 days)
- [ ] Option<T> enum implementation
- [ ] Result<T, E> enum implementation
- [ ] String type with interpolation
- [ ] Range types (exclusive/inclusive)

#### 4.2 Essential Modules (3-4 days)
- [ ] io module (print, read, files)
- [ ] mem module (allocators, utilities)
- [ ] collections (Vec, HashMap basics)
- [ ] math module (basic operations)

## Implementation Strategy

### Principles
1. **Test-Driven**: Write tests before implementation
2. **Incremental**: Small, working commits
3. **Clean Code**: KISS/DRY principles
4. **Documentation**: Update docs with each change

### Daily Workflow
1. Pick task from current phase
2. Write/update tests
3. Implement feature
4. Ensure all tests pass
5. Update documentation
6. Commit with clear message
7. Push to remote

### Success Metrics
- [ ] All lang.md examples compile and run
- [ ] 100% test coverage maintained
- [ ] No if/else in user code (only ? operator)
- [ ] Comptime evaluation functional
- [ ] @std namespace working
- [ ] Type errors caught before codegen

## Next Immediate Tasks (Today)
1. [ ] Start enforcing `=` syntax for functions
2. [ ] Implement `::=` operator for mutable variables
3. [ ] Begin removing if/else support
4. [ ] Update parser tests for new syntax
5. [ ] Create migration guide for syntax changes

## Notes
- Parser already supports most features, need to enforce them
- Keep backward compatibility during transition
- Focus on lang.md spec compliance
- Maintain all existing tests passing
- Document breaking changes clearly

## Resources
- lang.md: Complete language specification
- .agent/global_memory.md: Quick syntax reference
- .agent/todos.md: Current task tracking
- examples/zen_spec_demo.zen: Reference implementation