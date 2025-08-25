# Zen Language Global Memory

## Project Overview
Zen is a modern systems programming language with:
- **NO if/else keywords** - all conditionals use ? operator
- Pattern matching with unified ? operator 
- Functions use = syntax: `name = (params) returnType { ... }`
- Variables: := (immutable) and ::= (mutable)
- Compile-time metaprogramming with comptime
- Error handling via Result/Option types (no exceptions)
- LLVM backend for native code generation
- Minimal keyword design philosophy

## Key Language Features (from lang.md spec)

### 1. Pattern Matching (NO if/else!)
```zen
// All conditionals use ? operator
value ? | pattern => result
       | pattern -> binding => result  // with destructuring
       | _ => default
```

### 2. Variable Declaration
- `name := value` - immutable, inferred type
- `name ::= value` - MUTABLE, inferred type  
- `name: T = value` - immutable, explicit type
- `name:: T = value` - MUTABLE, explicit type

### 3. Functions
```zen
functionName = (param: Type) ReturnType {
    // body
}
```

### 4. Loops (single keyword)
```zen
loop condition { }  // while-like
loop item in collection { }  // for-each
```

### 5. Structs & Enums
```zen
Person = { name: string, age:: int }  // :: for mutable field
Action = | Stop | Go | Wait(int)
```

## Current Implementation Status
- ✅ Parser complete with all major features
- ✅ Pattern matching codegen working
- ✅ Functions with new syntax
- ✅ Variables (mutable/immutable)
- ✅ Structs, enums, arrays
- ✅ C FFI support
- ✅ LLVM backend
- 🚧 Type checker (separate from codegen)
- 🚧 Comptime evaluation
- 🚧 Generic instantiation
- 🚧 Behaviors/traits
- 📋 Standard library (@std namespace)

## Important Files
- `lang.md` - Complete language specification
- `src/parser/` - Parser implementation
- `src/codegen/llvm/` - LLVM code generation
- `examples/*.zen` - Example programs
- `tests/` - Comprehensive test suites

## Testing Commands
- Run all tests: `cargo test`
- Compile .zen file: `cargo run --bin zen file.zen`
- Run specific test: `cargo test test_name`

## Recent Updates (2025-08-25)
- Transitioned from lynlang to zen naming
- All tests passing with new function syntax
- Pattern matching fully implemented
- Working on aligning implementation with lang.md spec