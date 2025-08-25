# Zen Language Global Memory

## Language Specification
The authoritative language specification is in `/home/ubuntu/zenlang/lang.md`

## Key Syntax Features

### Pattern Matching
- Uses `?` operator: `scrutinee ? | pattern => expression`
- No `if/else` or `match` keywords
- Supports destructuring with `->` 
- Guards with `->` as well

### Function Declaration
- Syntax: `name = (params) ReturnType { body }`
- No `fn` keyword
- No `->` arrow for return type

### Variable Declaration
- `:=` for immutable inferred
- `::=` for mutable inferred
- `: T =` for immutable with type
- `:: T =` for mutable with type

## Recent Changes (Completed)
1. ✅ Removed all lynlang/lyn references
2. ✅ Updated parser to use `?` for pattern matching
3. ✅ Removed `match` keyword from lexer
4. ✅ Updated function syntax to use `=` and direct return type
5. ✅ Fixed all parser tests
6. ✅ Created working .zen examples

## Test Status
- Parser tests: ✅ All passing (29 tests)
- Lexer tests: Need to check
- Codegen tests: Need to update for new syntax

## Known Issues
- Some codegen tests may still use old syntax
- Type checker needs updates for new syntax
- LSP may need updates