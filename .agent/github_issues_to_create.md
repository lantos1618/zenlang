# GitHub Issues to Create

These issues should be created on https://github.com/lantos1618/lynlang/issues

## Issue 1: Implement Comptime Evaluation Engine
**Labels**: enhancement, high-priority
**Description**:
The parser for comptime blocks is complete, but we need to build the actual evaluation engine that executes code at compile-time.

### Current Status
- ✅ Parser support for comptime blocks and expressions
- ✅ Basic comptime module exists at src/comptime.rs
- ⚠️ Evaluator not integrated into compilation pipeline
- ⚠️ No actual compile-time execution happening

### Tasks
- [ ] Hook evaluator into compilation pipeline
- [ ] Implement comptime function execution
- [ ] Add compile-time type generation
- [ ] Enable compile-time assertions
- [ ] Test with real examples

---

## Issue 2: Build Generic Type Instantiation System
**Labels**: enhancement, high-priority
**Description**:
Implement full generic type support with monomorphization for zero-cost abstractions.

### Current Status
- ✅ Generic type parsing (List<T>, Map<K,V>)
- ✅ Generic function parsing (fn map<T, U>)
- ⚠️ No type instantiation engine
- ⚠️ No monomorphization in codegen

### Tasks
- [ ] Design generic parameter representation in AST
- [ ] Build type instantiation engine
- [ ] Implement monomorphization in LLVM codegen
- [ ] Add type parameter bounds
- [ ] Create comprehensive test suite

---

## Issue 3: Create Dedicated Type Checker Module
**Labels**: enhancement, high-priority
**Description**:
Build a robust type checking system separate from the parser and codegen.

### Tasks
- [ ] Create src/typechecker/ module
- [ ] Implement type inference engine
- [ ] Add type unification
- [ ] Build constraint solver
- [ ] Handle generic type checking
- [ ] Implement trait resolution

---

## Issue 4: Implement Trait/Behavior System
**Labels**: enhancement, high-priority
**Description**:
Design and implement a trait system for polymorphism and code reuse.

### Tasks
- [ ] Define trait syntax in lang.md
- [ ] Add trait parsing to parser
- [ ] Build trait resolution system
- [ ] Implement trait bounds
- [ ] Add dynamic dispatch support
- [ ] Create standard library traits

---

## Issue 5: Enhance Module System
**Labels**: enhancement, medium-priority
**Description**:
Build proper module and import system for code organization.

### Tasks
- [ ] Design import/export syntax
- [ ] Implement module resolution
- [ ] Add visibility rules (pub, priv)
- [ ] Create namespace support
- [ ] Build package management basics

---

## Issue 6: Improve Memory Management
**Labels**: enhancement, medium-priority
**Description**:
Add memory management primitives and smart pointers.

### Tasks
- [ ] Define allocator interface
- [ ] Implement reference counting
- [ ] Add smart pointer types
- [ ] Create arena allocators
- [ ] Build memory safety checks

---

## Issue 7: Build Standard Library Foundation
**Labels**: enhancement, medium-priority
**Description**:
Create core standard library with essential types and functions.

### Tasks
- [ ] Implement Vec<T> collection
- [ ] Add HashMap<K,V> 
- [ ] Create String type with operations
- [ ] Build I/O abstractions
- [ ] Add Result and Option types
- [ ] Implement iterators

---

## Issue 8: Add Async/Await Support
**Labels**: enhancement, low-priority
**Description**:
Implement async/await for concurrent programming.

### Tasks
- [ ] Design async syntax
- [ ] Add async function parsing
- [ ] Build runtime executor
- [ ] Implement Future trait
- [ ] Create async I/O primitives