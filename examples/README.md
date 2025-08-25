# Zen Language Examples

This directory contains comprehensive examples demonstrating the features of the Zen programming language.

## Examples Overview

### Basic Examples

1. **hello.zen** - Simple hello world program
   - Basic program structure
   - Module imports
   - Main function

2. **variables.zen** - Variable declarations and types
   - Immutable bindings (constants)
   - Mutable variables
   - Type annotations
   - String interpolation

3. **functions.zen** - Functions and UFCS
   - Function definitions
   - Default parameters
   - Uniform Function Call Syntax (UFCS)
   - Methods as free functions

### Control Flow

4. **pattern_matching.zen** - Pattern matching with `?` operator
   - No `if`/`else` keywords - unified `?` operator
   - Enum matching with destructuring
   - Range patterns
   - Guard conditions
   - Result type handling

5. **loops.zen** - Loop constructs
   - Conditional loops (while-like)
   - Range iteration
   - Array iteration
   - Break and continue
   - Labeled loops

### Data Structures

6. **structs_enums.zen** - Structs and enums
   - Struct definitions with defaults
   - Mutable fields
   - Nested structs
   - Enum variants with payloads
   - Anonymous struct payloads

### Advanced Features

7. **comptime.zen** - Compile-time metaprogramming
   - Compile-time evaluation blocks
   - Compile-time constants
   - Generic functions with comptime parameters
   - Type as values

8. **behaviors.zen** - Traits/Interfaces
   - Behavior definitions
   - Implementation blocks
   - Polymorphism
   - Multiple behavior implementation

9. **error_handling.zen** - Error handling patterns
   - Result<T, E> type
   - Option<T> type
   - Error propagation
   - Pattern matching for error handling
   - No exceptions - errors as values

## Running Examples

To compile and run an example:

```bash
cargo run --bin zen examples/hello.zen
```

## Language Philosophy

Zen follows these core principles:

1. **No hidden control flow** - All control flow is explicit
2. **Errors as values** - No exceptions, use Result/Option types
3. **Unified syntax** - Single `?` operator for all conditionals
4. **Compile-time power** - Strong metaprogramming with `comptime`
5. **Zero-cost abstractions** - High-level features compile to efficient code

## Key Syntax Elements

- **Function definition**: `name = (params) ReturnType { body }`
- **Variable binding**: `name := value` (immutable), `name ::= value` (mutable)
- **Pattern matching**: `value ? | pattern => result`
- **Loops**: Single `loop` keyword for all iteration
- **No if/else**: All conditionals use `?` operator
- **Destructuring**: Use `->` in patterns for binding

## Type System

- **Basic types**: `bool`, `i8`-`i64`, `u8`-`u64`, `f32`, `f64`, `string`
- **Pointer types**: `Ptr<T>` (raw), `Ref<T>` (managed)
- **Collections**: Arrays `[T]`, fixed arrays `[N]T`
- **Generic types**: `Stack<T>`, `Option<T>`, `Result<T,E>`
- **Type aliases**: Support for creating type synonyms

## Memory Management

- Explicit allocator interfaces
- No hidden allocations
- Manual memory management with safety features
- Future: Reference counting or ownership system

## Standard Library (Planned)

The `@std` namespace provides:
- `@std.core` - Compiler intrinsics
- `@std.build` - Build system interface
- Module imports via `build.import("module_name")`