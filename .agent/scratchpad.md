# Lynlang Development Scratchpad

## Quick Commands

### Run Tests
```bash
# All tests
cargo test

# Specific test module
cargo test generic
cargo test behavior
cargo test codegen

# With output
cargo test -- --nocapture
```

### Build & Check
```bash
# Check for errors
cargo check

# Build with all warnings
cargo build 2>&1 | grep warning | wc -l

# Build release
cargo build --release
```

### Code Quality
```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Find debug statements
grep -r "println!" src/ --include="*.rs"
grep -r "dbg!" src/ --include="*.rs"
```

## Current Working Notes

### Generic Type Integration Path
1. TypeInstantiator creates specialized types
2. Monomorphizer transforms whole program
3. LLVMCompiler needs to handle monomorphized output
4. Key integration point: `compile_generic_function` in compiler.rs

### Comptime Integration Path
1. ComptimeEvaluator in src/comptime/evaluator.rs
2. Hook needed in Compiler::compile (before LLVM)
3. Store evaluated values in symbol table
4. Replace comptime blocks with computed values

### Debug Statement Locations
- src/codegen/llvm/functions.rs - Multiple println! statements
- Check for any dbg! macros throughout codebase

### Unused Code Warnings Categories
1. Unused structs - May be for future features
2. Unused impl blocks - Check if actually needed
3. Unused functions - Determine if dead code or WIP

## Architecture Insights

### Type Flow
```
Parser -> AST -> TypeChecker -> TypeInstantiator -> Monomorphizer -> LLVM
```

### Compilation Pipeline
```
Source -> Lexer -> Parser -> AST -> TypeCheck -> Comptime -> Codegen -> LLVM -> Binary
```

### Key Integration Points
- TypeEnvironment: Tracks generic parameters
- TypeSubstitution: Maps type params to concrete types
- BehaviorResolver: Resolves trait implementations
- LLVMCompiler: Final code generation

## Test Coverage Areas
- Parser: Comprehensive (all features)
- Type System: Good (generics, behaviors)
- Codegen: Partial (basic features work)
- Comptime: Minimal (needs expansion)
- Integration: Needed for advanced features

## Performance Considerations
- Monomorphization can cause code bloat
- Vtable dispatch has runtime overhead
- Comptime evaluation should be cached
- LLVM optimization passes important

## Next Session Reminders
- Check context window size (aim for 100-140K)
- Run tests before any major changes
- Commit after each completed feature
- Update todos.md with progress
- Clean up temporary files