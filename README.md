# Zen Programming Language

A modern systems programming language designed for clarity, performance, and joy. Zen prioritizes explicit, consistent, and elegant syntax that composes into powerful patterns.

## Core Philosophy

- **Clarity over cleverness**: Code is read more often than written
- **Explicit is better than implicit**: No hidden control flow or allocations
- **Minimal but composable**: Small set of powerful primitives
- **Errors as values**: No exceptions, use Result/Option types
- **Powerful compile-time**: Deep metaprogramming capabilities

## Key Features

### ✅ Implemented
- Functions with `=` syntax: `name = (params) ReturnType { }`
- Variables (mutable/immutable) with `:=` and `::=` operators
- Basic types (integers, floats, strings, bool, void)
- Structs with mutable fields and defaults
- Enums with payloads and anonymous structs
- Conditional loops (`loop condition { }`)
- Fixed-size arrays `[T; N]` and slices
- Type aliases (`type Name = Type`)
- Generic type parsing
- Basic arithmetic and comparison operators
- Function calls and returns
- LLVM backend for native code generation

### 🚧 In Progress
- Pattern matching with unified `?` operator (parser complete, codegen WIP)
- Range expressions (`..` and `..=`)
- String interpolation with `$(expr)` syntax
- Loop iteration (`loop item in collection`)
- Compile-time evaluation (`comptime`)
- Type checker improvements
- Generic type instantiation and monomorphization
- Behaviors (traits/interfaces)
- Module system with `@std` namespace
- UFCS (Uniform Function Call Syntax)

### 📋 Planned
- Standard library (io, mem, collections, etc.)
- Memory management (Ptr<T>, Ref<T>, allocators)
- Error handling (Result<T,E>, Option<T>)
- Async/await
- Package management
- C FFI improvements

## Unique Syntax

Zen has a distinctive, keyword-minimal syntax:

```zen
// No 'if' or 'else' - unified ? operator for all conditionals
score ? | 90..=100 => "A"
       | 80..=89  => "B"
       | _        => "C"

// Pattern matching with destructuring using ->
result ? | .Ok -> value => process(value)
        | .Err -> msg => handle_error(msg)

// Single 'loop' keyword for all iteration
loop i in 0..10 { }           // Range iteration
loop condition { }             // While-like
loop item in items { }         // For-each

// Clean function syntax
add = (a: i32, b: i32) i32 { a + b }

// Variable declarations
PI := 3.14159                  // Immutable
counter ::= 0                  // Mutable
```

## Building

```bash
# Build the compiler
cargo build --release

# Run tests (all should pass)
cargo test

# Compile a Zen file
cargo run --bin zen examples/hello.zen
```

## Quick Start

New to Zen? Start here:
1. **[`examples/01_basics_working.zen`](examples/01_basics_working.zen)** - Simplest working example
2. **[`examples/02_functions_working.zen`](examples/02_functions_working.zen)** - Functions and calls
3. **[`examples/WORKING_FEATURES.md`](examples/WORKING_FEATURES.md)** - What currently works
4. **[`lang.md`](lang.md)** - Full language specification (v1.0)

## Examples

The `examples/` directory contains two categories:

### Working Examples (Current Implementation)
- **`01_basics_working.zen`** - Variables and arithmetic
- **`02_functions_working.zen`** - Function definitions and calls
- **`working_hello.zen`** - Minimal working program
- **`working_variables.zen`** - Variable declarations
- **`working_loops.zen`** - Basic loops
- **`WORKING_FEATURES.md`** - Complete list of working features

### Specification Examples (Future Features) 
- **`zen_spec_showcase.zen`** - Complete language specification demonstration (NEW)
- **`zen_master_showcase.zen`** - Comprehensive feature showcase
- **`01_hello_world.zen`** - Hello world per spec
- **`02_variables_and_types.zen`** - Full variable system
- **`03_pattern_matching.zen`** - Pattern matching with `?` operator
- **`04_loops.zen`** - All loop patterns per spec
- **`05_structs_and_methods.zen`** - Structs with UFCS
- Additional examples demonstrating planned features

## Project Structure

```
zen/
├── src/
│   ├── ast.rs              # Abstract syntax tree
│   ├── parser/             # Parser implementation
│   ├── codegen/            # LLVM code generation
│   ├── typechecker/        # Type checking (WIP)
│   ├── compiler.rs         # Main compiler logic
│   └── main.rs             # CLI entry point
├── examples/               # Example Zen programs
├── tests/                  # Integration tests
├── lang.md                 # Language specification
└── .agent/                 # Project metadata
```

## Development Status

- **Parser**: ✅ Core features implemented, including `?` pattern matching syntax
- **Code Generation**: ✅ LLVM backend working for basic features
- **Type System**: 🚧 Basic type checking, improvements in progress
- **Pattern Matching**: ✅ Parser complete (src/parser/expressions.rs:373-429), 🚧 Codegen WIP
- **Module System**: 📋 Specified with `@std` namespace, not yet implemented
- **Documentation**: ✅ Complete specification in lang.md
- **Examples**: ✅ 30+ example files demonstrating current and future features
- **Naming**: ✅ Consistently "zen" throughout (no "zena" references)

## Contributing

We welcome contributions! Check out:
- [GitHub Issues](https://github.com/anthropics/zen/issues) for bug reports and features
- `ROADMAP.md` for development priorities
- `STYLE_GUIDE.md` for code style guidelines

## Language Specification

See [`lang.md`](lang.md) for the complete language specification including:
- Detailed syntax rules
- Type system
- Memory model
- Standard library design

## License

[To be determined]

## Contact

Report issues at: https://github.com/anthropics/zen/issues