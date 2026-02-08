# Task 4: DefinitionLocation in TypeContext

## Key Findings
- `Span` struct has: `start`, `end`, `line`, `column` (no `end_line`/`end_column`)
- `Function` AST node has NO `span` field — cannot track function definition locations from declaration_checking
- `StructDefinition`, `EnumDefinition`, `TypeAlias`, `Constant`, `ModuleImport` all have `Option<Span>`
- `ImplBlock` has NO span — method locations cannot be tracked directly

## Pattern Used
- Added `collected_locations: HashMap<String, DefinitionLocation>` to TypeChecker (mirrors collected_variables pattern)
- `collect_location()` method on TypeChecker extracts from Span
- Transfer in `build_type_context()` to TypeContext
- Registration happens in `collect_declaration_types` (first pass)

## Pre-existing Issues
- `module_system/mod.rs` has 4 borrow checker errors (from parallel task changes)
- `lsp/document_store/document_lifecycle.rs` has missing field errors (from parallel task)
- These prevent `cargo test --lib` from compiling but are NOT from this task
- LSP diagnostics show zero errors on our 3 files

## Coverage
- Structs: ✅ tracked via struct_def.span
- Enums: ✅ tracked via enum_def.span
- Constants: ✅ tracked via Constant.span
- TypeAliases: ✅ tracked via type_alias.span
- ModuleImports: ✅ tracked via ModuleImport.span
- Functions: ❌ no span on Function AST node
- ImplBlock methods: ❌ no span on Function/ImplBlock

# Task 6: Smarter Caching

## Patterns
- `ModuleSystem.modules` internally changed from `HashMap<String, Program>` to `HashMap<String, CachedModule>` where `CachedModule` bundles `program + content_hash + insertion_order`
- `get_modules()` now returns owned `HashMap<String, Program>` (was `&HashMap`) — callers bind to variable before passing as `&`
- Borrow checker fix: use `contains_key` + boolean flag pattern instead of holding `&cached.program` reference across mutable operations

## Conventions
- `hash_content()` uses FNV-1a (same as in `types.rs`)
- Background worker thread owns `ModuleSystem` directly (no Arc<Mutex<>>)
- Cache eviction: oldest-first via monotonic insertion counter

## Gotchas
- Returning `&cached.program` from a `get()` call borrows self.modules for the function's return lifetime, preventing later `self.load_module()` or `self.insert_cached()`. Fix: check hash via `map().unwrap_or()`, then use `if !cache_valid { ... }` block.
- Doc-test failures in `type_aliases.rs` and `type_store.rs` are pre-existing (broken docstrings, not compilable code blocks)

## Task 5: TypeContext Definition Lookups

- `DefinitionLocation` uses 1-based line/column; LSP uses 0-based — need `saturating_sub(1)`
- `DefinitionLocation.file` is `None` for same-file defs (typechecker doesn't track file path)
- Keys in `definition_locations` are bare symbol names: struct names, enum names, function names, type alias names
- Keys for scoped variables use `"func::var"` format — not in `definition_locations`, only in `variables`
- `collect_location` is called from `declaration_checking.rs` for structs, enums, functions, module aliases, type aliases
- TypeContext lookup inserted after qualified name resolution (std/module paths) but before stdlib imports and text fallback
- All 19 existing navigation tests pass unchanged

## Task 7: Remove Redundant Text Fallback Paths

### What Was Removed
1. **resolve_text_fallback()** - Completely removed from definition.rs
   - Was the last fallback in the 10-step resolution chain
   - No longer needed because TypeContext handles most symbols

2. **find_symbol_definition_in_content() calls in stdlib lookups** - Removed 2 calls
   - Line 261 in resolve_member_in_module() - removed file-based fallback
   - Line 443 in resolve_symbol_from_stdlib() - removed file-based fallback
   - These were fallbacks for unindexed stdlib files, but document store now covers all cases

### What Was Kept (and Why)
1. **find_symbol_definition_in_content() function** - KEPT in utils.rs
   - Still used in 2 legitimate places:
     a. resolve_local_variable() - TypeContext doesn't track local variables (only functions, structs, enums, methods)
     b. handle_definition_hover() - Fallback for hover on incomplete/invalid code

2. **extract_symbols_text_fallback()** - KEPT in symbol_extraction.rs
   - Called when AST parsing fails (line 20)
   - Essential for LSP to work with incomplete code while user is typing
   - Not a "hot path" fallback - only used when parsing fails

### Key Insight
TypeContext (from Task 4) tracks definition locations for:
- Functions
- Structs
- Enums
- Methods
- Type aliases
- Import aliases

But NOT for:
- Local variables (checked typechecker/declaration_checking.rs - no variable location registration)
- Symbols in unparseable code

This is why some text-based fallbacks must remain.

### Test Results
All test suites pass:
- cargo test --lib: 148 passed
- cargo test --test lsp_navigation_tests: 19 passed
- cargo test --test lsp_analysis_tests: 41 passed
- cargo test --test lsp_completion_tests: 17 passed
- cargo test --test behavioral_tests: 39 passed
- cargo test --test ptr_ref_tests: 11 passed

### Files Modified
- src/lsp/navigation/definition.rs - removed resolve_text_fallback() and 2 stdlib fallback calls
