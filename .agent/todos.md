# Zen Language Project - TODOs

## Completed ✅
- [x] Fix self-hosted lexer syntax for parser compatibility
- [x] Verify all printf/puts tests capture and verify output  
- [x] Debug and fix failing vector test
- [x] Review project documentation and .agent files
- [x] Confirm testing infrastructure properly captures output
- [x] Fix segmentation fault in codegen tests (RESOLVED)
  - Reverted problematic changes in literals.rs
  - All tests passing again
- [x] Complete string interpolation implementation (DONE)
  - Fixed typechecker to recognize StringInterpolation
  - Fixed identifier loading for string pointers
  - Works correctly when stored in variables

## High Priority 🔴
1. [ ] Implement comptime execution system
   - Framework exists but not connected
   - Need to connect to compiler pipeline
   - Required for build system integration
   
2. [ ] Fix loop mutability issues
   - Cannot reassign variables in loops
   - Blocks many algorithm implementations

## Medium Priority 🟡
3. [ ] Complete self-hosted parser implementation
   - Basic structure created in stdlib/parser.zen
   - Need to implement actual parsing logic
   - Need to handle all AST node types
   
4. [ ] Write comprehensive stdlib modules in Zen
   - io.zen - Input/output operations
   - fs.zen - File system operations  
   - net.zen - Network operations (TCP/UDP)
   - collections.zen - Data structures
   - algorithms.zen - Common algorithms
   - mem.zen - Memory management

## Low Priority 🟢  
5. [ ] Create bootstrap process for self-hosting
   - Use self-hosted lexer/parser to compile Zen
   - Create multi-stage build script
   - Test bootstrap compilation
   
6. [ ] Clean up unused code warnings
   - Remove dead code or mark as used
   - Fix all compiler warnings
   
7. [ ] Add missing AST node types
   - Range expressions
   - String concatenation operator
   - Additional pattern matching cases
   
8. [ ] Improve loop syntax support
   - Allow function calls in loop conditions
   - Full iterator protocol support

## Technical Debt
- Lexer currently minimal - needs full implementation when parser supports more syntax
- Some AST types defined but never constructed  
- Warning about unused variables in parser code
- Many unused functions in codegen (might be needed later)