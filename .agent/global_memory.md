# Zen Language Global Memory

## Project Overview
Zen is a modern systems programming language written in Rust. The compiler generates LLVM IR for native code generation.

## Key Components
- **Parser**: Located in `src/parser/` - Implements recursive descent parser
- **Lexer**: `src/lexer.rs` - Tokenization 
- **AST**: `src/ast.rs` - Abstract syntax tree definitions
- **Type System**: `src/type_system/` and `src/typechecker/` - Type checking and inference
- **Code Generation**: `src/codegen/llvm/` - LLVM backend
- **Compile-time**: `src/comptime.rs` - Compile-time evaluation
- **LSP**: `src/lsp/` - Language server protocol support

## Language Features (from lang.md)
### Core Syntax
- **Functions**: `name = (params) ReturnType { }` (NO `fn`, `::`, or `->` keywords)
- **Variables**: 
  - Immutable: `name := value` or `name: Type = value`
  - Mutable: `name ::= value` or `name:: Type = value`
- **NO if/else**: All conditionals use `?` operator with pattern matching
- **Pattern matching**: `scrutinee ? | pattern => expression`
- **Destructuring**: Use `->` in patterns for binding/guards
- **Single loop construct**: `loop` for all iteration patterns
- **Comptime**: Compile-time metaprogramming with `comptime` blocks
- **Errors as values**: `Result<T,E>` and `Option<T>` instead of exceptions
- **Behaviors**: Traits/interfaces for polymorphism
- **UFCS**: Uniform function call syntax

## Current Status
- ✅ Parser fully implements lang.md specification
- ✅ All lynlang references converted to zen
- ✅ Examples follow the spec correctly
- ✅ Tests exist for all major features (145+ tests passing)

## Build Commands
```bash
cargo build --release  # Build compiler
cargo test            # Run tests
cargo run -- file.zen # Compile a zen file
```