# Generic Types Implementation Plan

## Overview
Implement full generic type support including type parameters, instantiation, and monomorphization in the lynlang compiler.

## Current State
- AST has a `Generic` type variant with name and type_args
- Parser treats unknown types as Generic with empty type_args
- No type parameter support in Function/Struct declarations
- No monomorphization in codegen

## Implementation Steps

### Phase 1: AST Extensions
1. Add type_params field to Function struct
2. Add type_params field to StructDeclaration 
3. Add type_params field to EnumDeclaration
4. Add TypeParameter struct with name and optional constraints

### Phase 2: Parser Updates
1. Parse type parameters in function declarations: `fn foo<T, U>(x: T) -> U`
2. Parse type parameters in struct declarations: `struct Box<T> { value: T }`
3. Parse type parameters in enum declarations: `enum Option<T> { Some(T), None }`
4. Parse type arguments in type expressions: `Box<i32>`, `Option<String>`
5. Parse where clauses for constraints (future)

### Phase 3: Type Checking
1. Create TypeEnvironment to track type parameters in scope
2. Implement type substitution for generic types
3. Validate type arguments match type parameters
4. Infer type arguments from usage where possible

### Phase 4: Monomorphization in Codegen
1. Track all generic instantiations needed
2. Generate specialized versions for each concrete type combination
3. Mangle names for specialized functions/types
4. Update call sites to use specialized versions

### Phase 5: Testing
1. Basic generic functions
2. Generic structs
3. Generic enums
4. Nested generics
5. Type inference

## Files to Modify
- src/ast.rs - Add type_params fields
- src/parser/declarations.rs - Parse type parameters
- src/parser/types.rs - Parse type arguments
- src/codegen/llvm/mod.rs - Add monomorphization
- src/codegen/llvm/types.rs - Handle generic type instantiation

## Success Criteria
- Can define and use generic functions
- Can define and use generic structs/enums
- Proper type checking of generic code
- Efficient monomorphized code generation
- All existing tests still pass