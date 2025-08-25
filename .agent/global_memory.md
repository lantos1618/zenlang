# Zen Language Global Memory

## Project Overview
- **Language:** Zen - A modern systems programming language
- **Implementation:** Rust-based compiler with LLVM backend
- **Specification:** lang.md defines the language spec (v1.0 conceptual)
- **Current State:** Core features implemented, aligning with lang.md spec

## Key Features (from lang.md)
1. No `if`/`else` keywords - uses `?` operator for all conditionals
2. Unified declaration syntax with `:=` (immutable) and `::=` (mutable)
3. Pattern matching with `?` and `->` for destructuring
4. `@std` namespace for core functionality
5. Behaviors (traits/interfaces) for polymorphism
6. Compile-time metaprogramming with `comptime`
7. Error handling as values (Result/Option types)
8. Single `loop` keyword for all iteration

## Project Structure
- `/src` - Rust compiler implementation
  - `/lexer.rs` - Tokenization
  - `/parser.rs` - AST generation
  - `/typechecker/` - Type checking
  - `/codegen/` - LLVM code generation
- `/examples` - Example .zen files
- `/tests` - Test files
- `/zen_test` - Additional test cases

## Recent Work (from git log)
- Completed zen language maintenance and documentation
- Aligned implementation with lang.md specification
- Updated all tests to match spec
- Added comprehensive implementation plan

## Naming Convention
- Must use "zen" consistently throughout codebase
- File extension: `.zen`
- Entry point: `main` function