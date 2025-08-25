# Zen Language - Currently Working Features

## Compiler Status
The zen compiler is in active development. This document lists what currently works vs what's specified in lang.md.

## Working Features ✅

### 1. Basic Syntax
- Function declarations: `name = (params) returnType { body }`
- Variable declarations:
  - Immutable: `name := value`
  - Mutable: `name ::= value`
- Comments: `// single line`
- Return statements

### 2. Types
- Basic types: `i32`, `i64`, `f32`, `f64`, `bool`
- Type annotations: `name: Type`

### 3. Expressions
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical: `&&`, `||`, `!`
- Function calls: `func(arg1, arg2)`
- Variable references

### 4. Control Flow (Partial)
- Basic if/else (old syntax, not spec compliant)
- Basic loops (old syntax, not spec compliant)

## Not Yet Working ❌

### 1. Language Spec Features
- Pattern matching with `?` operator
- Loop variations (`loop condition`, `loop item in collection`)
- @std namespace and imports
- String interpolation `$(expr)`
- Comptime blocks and execution

### 2. Type System
- Structs (parsing works, codegen incomplete)
- Enums
- Generics
- Result<T,E> and Option<T>
- Type inference improvements

### 3. Advanced Features
- Behaviors (traits)
- UFCS (Uniform Function Call Syntax)
- Async/await
- Memory management/allocators

## Example Programs

### Working Example 1: Basic Math
```zen
main = () i32 {
    x := 10
    y := 20
    result := x + y
    return result
}
```

### Working Example 2: Functions
```zen
add = (a: i32, b: i32) i32 {
    return a + b
}

main = () i32 {
    sum := add(5, 3)
    return sum
}
```

### Working Example 3: Variables
```zen
main = () i32 {
    // Immutable
    x := 42
    
    // Mutable
    counter ::= 0
    counter = counter + 1
    
    return x + counter
}
```

## Testing
Run examples with:
```bash
./target/debug/zen examples/01_basics_working.zen
```

## Development Status
- Parser: ~60% complete
- Type checker: ~40% complete  
- Code generator: ~30% complete
- Standard library: 0% complete

The compiler can handle basic procedural programming but lacks most advanced features from the language specification.