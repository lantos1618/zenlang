# Pattern Matching Codegen Implementation Plan

## Current State
- Parser: ✅ Complete with Pattern AST nodes
- Codegen: ❌ Only handles simple boolean conditions

## Patterns to Support
1. **Literal patterns**: `0`, `"hello"`, `true`
2. **Identifier patterns**: `x` (binds value)
3. **Wildcard pattern**: `_`
4. **Range patterns**: `1..10`, `1..=10`
5. **Or patterns**: `1 | 2 | 3`
6. **Struct patterns**: `Point { x, y }`
7. **Enum patterns**: `Some(value)`, `None`
8. **Guard patterns**: `x if x > 0`

## Implementation Strategy

### Phase 1: Basic Pattern Matching
1. Literal matching with integers
2. Wildcard pattern
3. Identifier binding
4. Multiple arms with fallthrough

### Phase 2: Advanced Patterns
1. Range patterns
2. Or patterns
3. Guards

### Phase 3: Complex Types
1. Struct destructuring
2. Enum variants

## Code Structure
```rust
fn compile_pattern_match(
    scrutinee: &Expression,
    arms: &[ConditionalArm]
) -> Result<BasicValueEnum, CompileError> {
    // 1. Evaluate scrutinee once
    // 2. Create entry block and exit block
    // 3. For each arm:
    //    a. Generate pattern test
    //    b. If match, execute body and jump to exit
    //    c. If no match, try next arm
    // 4. Create phi node at exit for result
}
```