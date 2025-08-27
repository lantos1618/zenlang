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

## Current Limitations
1. **Loop Syntax**: Cannot use function calls directly in loop conditions
2. **String Interpolation**: Syntax defined but codegen incomplete
3. **Comptime**: Framework exists but needs full integration
4. **Self-hosting**: Lexer and parser need more work for full bootstrap

## Important Files
- `/home/ubuntu/zenlang/stdlib/lexer.zen` - Minimal self-hosted lexer
- `/home/ubuntu/zenlang/stdlib/parser.zen` - Self-hosted parser (in progress)
- `/home/ubuntu/zenlang/.agent/zen_language_reference.md` - Language reference

## Testing Strategy
- All printf/puts tests verify actual output using ExecutionHelper
- Tests use assert_stdout_contains() and assert_exit_code()
- 100% test pass rate maintained

## Next Major Milestones
1. Complete self-hosted parser implementation
2. Implement string interpolation codegen
3. Integrate comptime execution
4. Create bootstrap process
5. Write comprehensive stdlib in Zen