# Zen Language Reference

## Overview
Zen is a modern systems programming language designed for clarity, performance, and joy. It features a minimal but composable syntax with powerful compile-time metaprogramming capabilities.

## Key Language Features (from lang.md)

### Core Philosophy
- **No Keywords Philosophy**: Minimal set of composable primitives instead of 30-50+ keywords
- **Pattern Matching Everything**: The `?` operator unifies conditionals, switches, and destructuring
- **Explicit Error Handling**: Errors as values using Result<T,E> and Option<T>
- **Compile-time Metaprogramming**: Powerful `comptime` system for zero-cost abstractions

### Syntax Summary

#### Variable Declaration
| Syntax | Mutability | Type | Description |
|--------|------------|------|-------------|
| `name := value` | Immutable | Inferred | Primary local constant |
| `name ::= value` | **Mutable** | Inferred | Primary local variable |
| `name: T = value` | Immutable | Explicit | Constant with explicit type |
| `name:: T = value` | **Mutable** | Explicit | Variable with explicit type |

#### Functions
```zen
// Simple function
add = (a: i32, b: i32) i32 { a + b }

// With default parameters
greet = (name: string, prefix: string = "Hello") void {
    io.print("$(prefix), $(name)!")
}
```

#### Pattern Matching (`?` operator)
```zen
// Replaces if/else/switch/match
x ? | val -> val > 0 => "positive"
    | val -> val < 0 => "negative"
    | _ => "zero"
```

#### Loop Construct
```zen
loop condition { /* while-like */ }
loop { /* infinite loop */ }
// For range and collection iteration, use functional methods:
range(0, 10).loop(i -> { })   // Range iteration
items.loop(item -> { })        // Collection iteration (future)
```

#### Data Structures
```zen
// Structs (Product Types)
Person = {
    name: string,
    age: int,
    email:: Option<string> = None,
}

// Enums (Sum Types)
Result<T, E> = 
    | Ok(value: T)
    | Err(error: E)
```

#### Module System
```zen
comptime {
    core := @std.core      // Compiler intrinsics
    build := @std.build    // Build system interface
    io := build.import("io")
}
```

### Major Language Components

1. **@std Namespace**: Bootstrap mechanism for compiler intrinsics and modules
2. **Behaviors**: Trait/interface system for polymorphism
3. **Comptime**: Compile-time execution and metaprogramming
4. **Memory Management**: Ptr<T>, Ref<T>, and allocator patterns
5. **String Interpolation**: `$(expression)` syntax
6. **UFCS**: Uniform Function Call Syntax
7. **Async/Await**: Task<T> type and async runtime

## File Format
- **Extension**: `.zen`
- **Encoding**: UTF-8
- **Comments**: `// single-line`
- **Entry Point**: `main = () void { }`

## Standard Library Modules (Conceptual)
- `@std.core` - Type information, memory operations
- `@std.build` - Module import system
- `@std.io` - Input/output operations
- `@std.mem` - Memory management
- `@std.math` - Mathematical functions
- `@std.collections` - Data structures
- `@std.net` - Network operations
