# Generic Types Design for Lynlang

## Overview
Implementing full generic type support with type parameters, instantiation, and monomorphization.

## Syntax Design

### Type Parameters
```lyn
// Generic struct
struct List<T> = {
    items: [T],
    size: u64,
}

// Generic function
fn map<T, U>(list: List<T>, f: fn(T) -> U) -> List<U> = {
    // implementation
}

// Generic enum
enum Option<T> = {
    Some(T),
    None,
}
```

### Type Instantiation
```lyn
// Explicit instantiation
let numbers: List<i32> = List { items: [], size: 0 }

// Type inference
let opt = Option::Some(42)  // Option<i32> inferred
```

### Constraints (Future)
```lyn
fn sum<T: Numeric>(list: List<T>) -> T = {
    // T must implement Numeric trait
}
```

## Implementation Plan

### Phase 1: Parser Enhancement ✅
- [x] Basic type parameter parsing in structs
- [ ] Type arguments in type expressions (List<T>)
- [ ] Generic function parsing
- [ ] Generic enum parsing

### Phase 2: Type System
- [ ] Type parameter tracking
- [ ] Type substitution
- [ ] Type inference for generics
- [ ] Constraint checking

### Phase 3: Codegen (Monomorphization)
- [ ] Track instantiated types
- [ ] Generate specialized versions
- [ ] Link instantiations

## AST Changes Needed

1. Enhance `AstType::Generic` to handle type arguments properly
2. Add type parameter support to functions
3. Track generic instantiations in a registry

## Test Cases

1. Basic generic struct
2. Generic function
3. Nested generics (List<Option<T>>)
4. Type inference
5. Multiple type parameters
6. Generic methods