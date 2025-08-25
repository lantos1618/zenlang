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
- Structs and enums with pattern matching
- Pattern matching with unified `?` operator (no if/else keywords)
- Loop constructs (conditional and iterative)
- Fixed-size arrays `[T; N]` and slices
- Type aliases (`type Name = Type`)
- Generic type parsing
- Range expressions (exclusive `..` and inclusive `..=`)
- String interpolation with `$(expr)` syntax
- C FFI (Foreign Function Interface)
- LLVM backend for native code generation

### 🚧 In Progress
- Compile-time evaluation (`comptime`)
- Type checker (separate from codegen)
- Generic type instantiation and monomorphization
- Behaviors (traits/interfaces)
- Module system with `@std` namespace

### 📋 Planned
- Standard library (Vec, HashMap, etc.)
- Memory management (allocators, references)
- Async/await
- Package management

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

## Examples

See the [`examples/`](examples/) directory for comprehensive examples:
- `zen_spec_demo.zen` - Complete demonstration of lang.md specification
- `hello.zen` - Basic hello world
- `variables.zen` - Variable declarations
- `functions.zen` - Functions and UFCS
- `pattern_matching.zen` - Pattern matching with `?`
- `structs_enums.zen` - Data structures
- `loops.zen` - Loop constructs
- `comptime.zen` - Compile-time evaluation
- `behaviors.zen` - Traits/interfaces
- `error_handling.zen` - Result/Option types

## Project Structure

```
zenlang/
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

- **Parser**: ✅ Complete with all major features
- **Code Generation**: ✅ Working for core features
- **Type System**: 🚧 Being separated from codegen
- **Test Coverage**: ✅ 100% passing (224 tests across 36 suites)

## Contributing

We welcome contributions! Check out:
- [GitHub Issues](https://github.com/anthropics/zenlang/issues) for bug reports and features
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

Report issues at: https://github.com/anthropics/zenlang/issues