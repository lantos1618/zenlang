# Draft: LSP-Compiler Deduplication

## Requirements (confirmed)
- User wants "all of the above" — full sweep of LSP/compiler duplication
- Symbol resolution overhaul, kill inference fallbacks, move analysis into compiler

## Key Research Findings

### TypeContext Architecture
- TypeContext stores types ONLY (no file/line/column positions)
- SymbolInfo (extracted from AST) already HAS position data
- TypeContext is populated ASYNCHRONOUSLY — timing window where it's unavailable
- Fallbacks exist INTENTIONALLY for this timing window (not legacy code)

### What's Actually Duplicated
1. **Variable type inference** — 2 implementations (hover/inference.rs, variable_extraction.rs)
2. **Type name extraction** — 3 implementations (semantic_completion.rs, hover/mod.rs, type_query.rs)
3. **Expression type inference for pattern checking** — uses callback pattern instead of TypeContext
4. **Text-based symbol search** — navigation/utils.rs (BUT: only 20% of lookups, rest is already AST-based)

### What's NOT Actually Duplicated (LSP-specific lints)
- Pattern exhaustiveness checking — compiler doesn't enforce this, LSP-specific lint
- Allocator validation — compiler doesn't enforce this, LSP-specific lint
- These SHOULD move to compiler (both compiler and LSP benefit), but they're not "duplication"

### Text-Based Search Reality
- LSP is already 80% AST-based for symbol resolution
- Text search is fallback for: local variables, parse failures, cross-file search
- TypeContext lacks position data → can't replace text search with TypeContext
- SymbolInfo has positions → could replace text search with SymbolInfo lookups
- BUT: local variables aren't in SymbolInfo (only module-level symbols)

## Technical Decisions
- Extend TypeContext with definition_locations: HashMap<String, (Url, Range)> → enables compiler-side position tracking
- Add AstType::base_name() → eliminates 3 duplicate type name extraction implementations
- Unify variable type inference → single implementation in type_query.rs
- Refactor pattern_checking to accept TypeContext directly instead of callback pattern
- Move pattern exhaustiveness + allocator validation into compiler's typechecker

## Scope Boundaries
- INCLUDE: All duplication points identified
- INCLUDE: Moving LSP lints into compiler
- INCLUDE: Extending TypeContext with position data
- EXCLUDE: Incremental parsing/typechecking (optimization, not dedup)
- EXCLUDE: Removing text-based fallback entirely (still needed for parse failures)
- EXCLUDE: Module loading caching (optimization, not dedup)

## Open Questions
- None — research is comprehensive, ready for plan generation

## Test Strategy
- Infrastructure exists: YES (cargo test)
- Automated tests: Tests-after (existing tests verify correctness)
- 143 lib tests must continue passing
- LSP handlers must continue working (test via cargo check)
