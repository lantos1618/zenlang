# Zen Programming Language

**The World's First AI-Native Systems Programming Language**

A revolutionary programming language with **ZERO KEYWORDS**. All control flow through pattern matching (`?`), UFC (Uniform Function Call), and powerful metaprogramming.

> *"No keywords. Pure expression. Allocator-driven concurrency."*

---

## Project Status: Late Alpha (90% Core Complete)

The Zen compiler is functional with all core language features working. The project has a comprehensive test suite and full LSP support for IDE integration.

### What Works

- **Zero-keyword syntax** - Pattern matching with `?` replaces all conditionals
- **All 6 variable forms** - Immutable/mutable, typed/inferred declarations
- **Type system** - Structs, enums, generics, Option<T>, Result<T,E>
- **UFC** - Uniform Function Call for method chaining
- **Error handling** - `.raise()` for error propagation
- **Collections** - Vec<T>, String with allocator support
- **Behaviors** - Structural trait system
- **I/O** - Syscall-based file and network I/O (Linux x86-64)
- **LSP** - Full IDE support with semantic completion, hover, go-to-def, etc.
- **25+ intrinsics** - Memory, pointers, syscalls, atomics

### In Progress

- Module system improvements for cross-boundary generics
- Iterator combinators (map, filter, collect)
- First-class closure support

### Planned

- Cross-platform (macOS, Windows)
- Package manager
- Self-hosting compiler

---

## Quick Start

```bash
# Build the compiler
cargo build --release

# Run a Zen program
./target/release/zen examples/showcase.zen

# Compile to executable
./target/release/zen examples/hello.zen -o hello
./hello

# Run test suite
cargo test --all
```

---

## Language at a Glance

### Variables (No Keywords)

```zen
x = 10              // Immutable, inferred
x ::= 10            // Mutable, inferred
x : i32 = 10         // Immutable, typed
x :: i32 = 10        // Mutable, typed
```

### Pattern Matching (Replaces if/else/match)

```zen
value ?
    | Some(x) { use(x) }
    | None { handle_empty() }

status ?
    | .Active { process() }
    | .Inactive { wait() }
```

### Functions and UFC

```zen
add = (a: i32, b: i32) i32 { return a + b }

// Both work:
result = add(5, 3)
result = 5.add(3)
```

### Structs and Enums

```zen
Point: { x: f64, y: f64 }
Color: Red, Green, Blue
Option<T>: Some: T, None
```

### Error Handling

```zen
load = (path: string) Result<Data, Error> {
    file = File.open(path).raise()  // Early return on error
    return Ok(file.read())
}
```

### Memory (Zig-Style Allocators)

```zen
allocator = GPA.new()
vec = Vec<i32>.new(allocator)
vec.mut_ref().push(42)
vec.mut_ref().free()
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [docs/OVERVIEW.md](docs/OVERVIEW.md) | Complete language overview |
| [docs/QUICK_START.md](docs/QUICK_START.md) | Getting started guide |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Compiler architecture |
| [docs/INTRINSICS_REFERENCE.md](docs/INTRINSICS_REFERENCE.md) | Intrinsics reference |
| [LANGUAGE_SPEC.zen](LANGUAGE_SPEC.zen) | Language specification |

For contributors:
- [docs/design/STDLIB_DESIGN.md](docs/design/STDLIB_DESIGN.md) - Standard library design
- [docs/ROADMAP.md](docs/ROADMAP.md) - Current roadmap

---

## Project Structure

```
zenlang/
+-- src/                # Rust compiler source
|   +-- parser/         # Syntax analysis
|   +-- typechecker/    # Type inference
|   +-- codegen/llvm/   # LLVM backend
|   +-- lsp/            # Language server
+-- stdlib/             # Standard library (Zen)
+-- tests/              # Test suite
+-- examples/           # Example programs
+-- docs/               # Documentation
+-- vscode-extension/   # VS Code integration
```

---

## Design Principles

1. **Zero Keywords** - Pattern matching for all control flow
2. **Explicit Over Implicit** - Allocators, pointers, errors all explicit
3. **UFC Everywhere** - Any function callable as method
4. **No Null** - Only Option<T> with Some/None
5. **Syscall-First** - Direct syscalls, minimal runtime

---

## Building

### Prerequisites

- Rust 1.75+
- LLVM 18.1
- Linux x86-64 (primary target)

### Commands

```bash
cargo build --release          # Build compiler
cargo test --all               # Run tests
cargo build --bin zen-lsp      # Build LSP
```

---

## Contributing

This project implements the specification in [LANGUAGE_SPEC.zen](LANGUAGE_SPEC.zen). All contributions must align with this specification.

---

## License

MIT
