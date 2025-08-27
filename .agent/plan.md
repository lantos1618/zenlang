# Loop Syntax Simplification Plan

## Objective
Remove range and iterator loop variants in favor of functional iteration:
- Remove: `loop i in 0..10 { }` (range iteration)
- Remove: `loop item in items { }` (for-each)  
- Keep: `loop condition { }` (while-like)
- Keep: `loop { }` (infinite)
- Add: `range(1,10).loop(i -> {})` functional style
- Add: `loop(condition -> {})` functional style

## Current State Analysis
The loop syntax has already been simplified in recent commits (see git log). The AST only has:
- `LoopKind::Infinite` for `loop { }`
- `LoopKind::Condition(expr)` for `loop condition { }`

Range and iterator loops have been removed from the parser.

## Tasks
1. ✅ AST already simplified (only Infinite and Condition variants)
2. ✅ Parser already simplified (parse_loop_statement only handles these two)
3. Need to add functional loop methods to stdlib:
   - Add `loop` method to iterators/ranges
   - Implement `range()` function that returns an iterable
4. Update examples to use new functional syntax
5. Clean up any remaining references to old loop syntax in docs/tests
6. Test the new implementation
7. Commit and merge changes

## Next Steps
1. Check stdlib for iterator/range implementations
2. Add functional loop methods
3. Update examples and tests
4. Clean up documentation
