# Zen Language Project - TODOs

## Completed ✅
- [x] Fix self-hosted lexer syntax for parser compatibility
- [x] Verify all printf/puts tests capture and verify output  
- [x] Debug and fix failing vector test
- [x] Run tests and verify all pass (100% pass rate achieved)
- [x] Commit and push changes

## In Progress 🚧
- [ ] Complete self-hosted parser implementation
  - Basic structure created in stdlib/parser.zen
  - Need to implement actual parsing logic
  - Need to handle all AST node types

## Pending 📋

### High Priority
- [ ] Implement string interpolation codegen
  - Syntax: `$(expression)`
  - Parser recognizes it but codegen incomplete
  
- [ ] Integrate comptime execution system
  - Framework exists
  - Need to connect to compiler pipeline
  
### Medium Priority  
- [ ] Write comprehensive stdlib modules in Zen
  - io.zen - Input/output operations
  - fs.zen - File system operations
  - net.zen - Network operations (TCP/UDP)
  - collections.zen - Data structures
  
### Low Priority
- [ ] Create bootstrap process for self-hosting
  - Use self-hosted lexer/parser to compile Zen
  - Create build script for bootstrap
  
- [ ] Improve loop syntax support
  - Allow function calls in loop conditions
  - Full iterator protocol support
  
- [ ] Add missing AST node types
  - Range expressions
  - String concatenation operator
  - Additional pattern matching cases

## Technical Debt
- Lexer currently minimal - needs full implementation when parser supports more syntax
- Some AST types defined but never constructed
- Warning about unused variables in parser code