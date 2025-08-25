# Zen Language Alignment Plan

## High Priority: Language Spec Alignment with lang.md

### Critical Differences Found:
1. **Pattern Matching Syntax**: Parser uses `match` keyword, but lang.md specifies `?` operator
2. **Function Declaration**: Need to verify `=` syntax for functions is implemented
3. **Variable Declaration**: Need to verify `::=` for mutable and `:=` for immutable

### Implementation Tasks:

#### 1. Update Parser for `?` Pattern Matching (HIGH PRIORITY)
- Replace `match` keyword with `?` operator
- Implement `scrutinee ? | pattern => expression` syntax
- Support `->` for destructuring and guards in patterns
- Update AST to handle new pattern matching syntax

#### 2. Verify Function Declaration Syntax
- Ensure functions use `name = (params) returnType { ... }` syntax
- No `fn` keyword should exist

#### 3. Verify Variable Declaration Syntax
- `:=` for immutable bindings (inferred type)
- `::=` for mutable bindings (inferred type)  
- `: T =` for immutable with explicit type
- `:: T =` for mutable with explicit type

#### 4. Create Working Examples
- Create .zen example files demonstrating all language features
- Test pattern matching with `?` operator
- Test function declarations
- Test variable declarations
- Test control flow

#### 5. Update Tests
- Update all test files to use new syntax
- Ensure tests pass with new parser

## Implementation Order:
1. Update lexer to recognize `?` as operator
2. Update parser to handle `?` pattern matching
3. Remove `match` keyword support
4. Update all tests
5. Create working examples
6. Update documentation