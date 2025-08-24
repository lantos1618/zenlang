# Lynlang Global Memory

## Project Overview
Lynlang is a Rust-inspired systems programming language with:
- Pattern matching
- Generic types with monomorphization
- Comptime evaluation
- Fixed-size arrays [T; N]
- Enum variants with :: syntax
- Type aliases (type Alias = Type)
- LLVM backend for code generation

## Key Syntax
- Function definition: `name :: (params) -> ReturnType { body }`
- Variable declaration: `name := value` (inferred) or `name : Type = value` (explicit)
- Enum variants: `EnumName::VariantName`
- Type alias: `type Name = Type` or `type Name<T> = Type<T>`

## Recent Achievements (2025-08-24)
- Fixed all test failures
- Implemented type alias support
- All tests passing (including comptime evaluation tests)
- Zero compiler warnings
- Cleaned up obsolete .agent files
- Fixed function syntax in comptime tests

## Next Priorities
1. Type alias resolution in type checker
2. Advanced comptime features
3. Module system design
4. Standard library (Vec, HashMap)

## Testing
- Run tests: `cargo test`
- Compile .lyn file: `cargo run --bin zen file.lyn`