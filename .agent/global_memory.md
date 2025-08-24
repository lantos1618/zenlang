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
- Fixed all test failures - **165 tests passing (100% pass rate)**
- Updated all test files to use new function syntax (:: and ->)
- Fixed parser tests, multi-backend tests, comptime tests, error recovery tests
- Implemented type alias support
- Parser complete with all major syntax features
- Pattern matching codegen working

## Next Priorities
1. Implement comptime evaluation engine (parser done, needs execution)
2. Create dedicated type checker module (currently mixed with codegen)
3. Implement generic type system with monomorphization
4. Design and implement trait/behavior system
5. Standard library (Vec, HashMap)

## Testing
- Run tests: `cargo test`
- Compile .lyn file: `cargo run --bin zen file.lyn`