# Zenlang Global Memory

## Project Status
- **Self-hosting**: Near completion (39/40 test suites passing)
- **Loop syntax migration**: COMPLETE - all old syntax removed
- **Standard library**: 29 modules, 12,144 lines of pure Zen code
- **Current branch**: master, 17 commits ahead
- **Recent fixes**: Test syntax updates for structs and pattern matching

## Key Facts
- No keywords philosophy - minimal composable primitives
- Pattern matching with `?` operator
- Functional loop syntax: `range().loop()` and `items.loop()`
- Complete stdlib written in Zen
- LLVM backend for production performance

## Current Focus
1. Fix function pointer test failure
2. Clean up temporary test files
3. Merge to main branch
4. Prepare for bootstrap compilation

## Important Paths
- Source: `/home/ubuntu/zenlang/src/`
- Standard Library: `/home/ubuntu/zenlang/stdlib/`
- Tests: `/home/ubuntu/zenlang/tests/`
- Examples: `/home/ubuntu/zenlang/examples/`