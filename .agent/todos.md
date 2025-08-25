# Zen Language TODOs

## High Priority - Language Spec Alignment
- [ ] **CRITICAL**: Align parser/codegen with lang.md spec
  - [ ] Change function syntax from `::` to `=` 
  - [ ] Implement mutable variables with `::=` operator
  - [ ] Remove if/else keywords, use only ? operator
  - [ ] Implement -> for pattern destructuring
  
## Current Session Tasks
1. ✅ Read lang.md spec 
2. ✅ Search for lynlang references (none found)
3. 🔄 Review and consolidate project information
4. [ ] Create working .zen examples matching spec
5. [ ] Run tests and fix failures
6. [ ] Update README with accurate syntax
7. [ ] Clean up unnecessary files
8. [ ] Git commit and push

## Implementation Priorities (80/20 rule)
1. **Parser Updates (40%)** - Match lang.md syntax exactly
2. **Codegen Updates (30%)** - Support new syntax in LLVM
3. **Examples (10%)** - Create clear, working examples
4. **Testing (20%)** - Ensure everything works

## Technical Debt
- Type checker needs separation from codegen
- Comptime evaluation engine incomplete
- Generic monomorphization not implemented
- Behaviors/traits system missing
- @std namespace bootstrap not implemented

## Notes
- Keep implementation simple and elegant (KISS/DRY)
- Work best at 40% context window (100-140k tokens)
- Use frequent git commits
- Clean up after completion