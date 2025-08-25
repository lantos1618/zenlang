# Zen Language - Global Memory

## Project Overview
Zen is a modern systems programming language designed for clarity, performance, and joy. It prioritizes explicit, consistent, and elegant syntax that composes into powerful patterns.

## Current Project State
- **Language Name**: zen (consistent throughout)
- **File Extension**: .zen
- **Package Name**: zen
- **Binary**: zen compiler, zen-lsp
- **Working Directory**: /home/ubuntu/zenlang/

## Core Language Features (from lang.md spec)
1. **No if/else keywords** - Uses `?` operator for pattern matching
2. **Single loop keyword** - `loop` for all iterations
3. **Unified assignment** - `=` for assignment, `:=` for immutable, `::=` for mutable
4. **@std namespace** - Bootstrap mechanism for compiler intrinsics
5. **Errors as values** - Result<T,E> and Option<T>
6. **UFCS** - Uniform Function Call Syntax
7. **Behaviors** - Traits/Interfaces
8. **Comptime** - Compile-time metaprogramming

## Current Implementation Status
### Working Features
- Basic function declarations
- Variable declarations (`:=` and `::=`)
- Basic types (i32, f64, bool, string)
- Return statements
- Basic arithmetic operations
- Function calls
- Struct definitions (partial)
- Pattern matching (partial)

### Not Yet Implemented
- @std namespace and imports
- Full pattern matching with `?` operator
- Behaviors/traits
- Comptime execution
- Error handling (Result/Option)
- String interpolation
- Async/await
- Memory management/allocators

## Key Files and Directories
- `/lang.md` - Language specification
- `/src/` - Compiler source code
- `/examples/` - Example zen programs
- `/tests/` - Test suite
- `/.agent/` - Meta information for maintenance

## Testing Commands
```bash
cargo build              # Build compiler
cargo test              # Run test suite
./target/debug/zen <file.zen>  # Run zen file
```

## Important Notes
- The compiler currently supports a subset of the language spec
- Working examples are in `examples/working_*.zen`
- Full spec examples need compiler updates to work