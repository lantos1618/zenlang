# Zen Language Project - Global Memory

## Project Overview
- **Language**: Zen - A modern systems programming language
- **Goal**: Achieve self-hosting capability with comprehensive standard library
- **Current Status**: 100% test pass rate (all 48 test suites passing)
- **Branch**: ragemode

## Key Achievements
1. **Test Coverage**: All tests passing including:
   - FFI tests with proper output verification
   - Self-hosted lexer parsing correctly
   - Vector implementation tests
   - Iterator and loop tests
   
2. **Self-Hosted Components**:
   - Minimal lexer implementation that parses correctly
   - Basic parser structure created
   - Standard library modules in Zen

3. **Language Features Working**:
   - Function declarations with `=` syntax
   - Variable declarations (`:=` and `::=`)
   - Pattern matching with `?` operator
   - Structs and generics
   - Arrays and pointers
   - External function declarations
   - Loops (with some syntax limitations)
   - String interpolation with `$(expr)` syntax (fully working)

## Current Limitations
1. **Loop Syntax**: Cannot reassign variables in loops (immutability)
2. **Void Functions**: Not fully supported - must return values
3. **Modulo Operator**: Not working correctly in all contexts
4. **Type System**: Some type inference issues with mixed int/float
5. **Struct Generics**: Generic struct types not monomorphizing correctly
6. **Module-level Constants**: Not supported, must use functions
7. **Comptime**: Framework exists but needs full integration
8. **Self-hosting**: Need more core features before stdlib can be written in Zen

## Important Files
- `/home/ubuntu/zenlang/stdlib/lexer.zen` - Minimal self-hosted lexer
- `/home/ubuntu/zenlang/stdlib/parser.zen` - Self-hosted parser (in progress)
- `/home/ubuntu/zenlang/.agent/zen_language_reference.md` - Language reference

## Testing Strategy
- All printf/puts tests verify actual output using ExecutionHelper
- Tests use assert_stdout_contains() and assert_exit_code()
- 100% test pass rate maintained

## Next Major Milestones
1. Fix loop mutability issues (allow mutable loop variables)
2. Implement void function support properly
3. Fix modulo operator parsing/codegen
4. Complete self-hosted parser implementation
5. Integrate comptime execution
6. Create bootstrap process
7. Write comprehensive stdlib in Zen