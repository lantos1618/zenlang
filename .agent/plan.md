# Zen Language Development Plan

## Current Sprint: Core Feature Completion
**Goal**: Bridge gap between working features and language specification
**Duration**: This session + next 2-3 sessions

### Phase 1: String Interpolation (TODAY)
1. Review existing parser for $(expr) syntax
2. Implement codegen in LLVM backend
3. Add comprehensive tests
4. Update working examples

### Phase 2: Loop Syntax Compliance
1. Update parser for spec-compliant syntax:
   - `loop condition { }` 
   - `loop item in items { }`
2. Update codegen for new syntax
3. Migrate all tests to new syntax
4. Update examples

### Phase 3: Enum Completion
1. Finish enum codegen (parsing already done)
2. Implement pattern matching on enums
3. Add discriminated union support
4. Test with Result/Option types

### Phase 4: Comptime Foundation
1. Design comptime execution context
2. Implement basic comptime blocks
3. Add compile-time function evaluation
4. Enable const generics

## Next Sprint: Standard Library Expansion
**Goal**: Build comprehensive std library in Zen
**Duration**: 3-4 sessions

### Collections Module
- Vec<T> - Dynamic arrays
- HashMap<K,V> - Hash tables
- Set<T> - Hash sets
- Deque<T> - Double-ended queue

### Memory Module
- Allocator trait
- Arena allocator
- Pool allocator
- Smart pointers

### Net Module
- TCP client/server
- UDP sockets
- HTTP basics
- URL parsing

## Future Sprint: Self-Hosting Preparation
**Goal**: Bootstrap compiler in Zen
**Duration**: 5-10 sessions

### Prerequisites
1. Complete standard library
2. Full generics support
3. Behaviors/traits system
4. File I/O complete

### Steps
1. Port lexer to Zen
2. Port parser to Zen
3. Port type checker to Zen
4. Create Zen->LLVM IR generator in Zen
5. Bootstrap compile

## Success Metrics
- 100% spec compliance for core features
- Standard library covers 80% use cases
- Self-hosted compiler passes all tests
- Performance within 2x of Rust version

## Risk Mitigation
- Frequent commits to avoid losing work
- Comprehensive testing at each step
- Keep Rust version as reference
- Document all design decisions