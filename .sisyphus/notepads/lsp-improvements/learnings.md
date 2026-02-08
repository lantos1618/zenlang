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
