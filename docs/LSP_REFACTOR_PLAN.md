# LSP Refactor: Replace String Parsing with Compiler SEMA

## Problem

The LSP (14K lines, 53 files) contains massive amounts of fragile string-based parsing
that duplicates what the compiler already provides. The compiler has industrial-grade
semantic analysis infrastructure (TypeStore, TypeContext, WellKnownTypes, StdlibTypeRegistry,
name_utils, intrinsics registry) that the LSP barely uses.

## Antipattern Summary

| Category | Count | Severity | Examples |
|----------|-------|----------|----------|
| String-based construct detection | 8+ | HIGH | `.starts_with("//")`, `.find(':')`, `.split('.')` to parse code |
| Hardcoded `@std` / type names | 10+ | MEDIUM | `"String"`, `"Option"`, `"Result"` magic strings |
| Diagnostic message parsing | 3+ | CRITICAL | `.contains("expected String")` to detect error type |
| Field/symbol extraction via split | 6+ | HIGH | `.split(',')` to find struct fields |
| Path-based file detection | 8+ | MEDIUM | `.contains("stdlib")` to detect stdlib files |
| Fallback string parsing (dead code) | 50+ lines | HIGH | Duplicates parser that already succeeded |
| Manual type inference | 177 lines | HIGH | Reimplements typechecker in hover/expressions.rs |
| Reference kind detection | 30+ lines | MEDIUM | `starts_with("+=")` for write detection |

## Compiler APIs the LSP Should Use

### Tier 1: Already Available, Underused

| API | Location | What It Provides |
|-----|----------|------------------|
| `TypeContext` | `src/type_context.rs` | Variables, functions, structs, enums, methods, locations |
| `TypeStore` | `src/type_system/type_store.rs` | 45+ query methods for all type info |
| `WellKnownTypes` | `src/well_known.rs` | Option/Result/Ptr detection without magic strings |
| `StdlibTypeRegistry` | `src/stdlib_types.rs` | All stdlib types with source locations |
| `name_utils` | `src/name_utils.rs` | `split_method_path()`, `strip_generics()`, `method_key()` |
| `Intrinsics` | `src/intrinsics.rs` | All builtins with signatures, docs, categories |
| `AstType::Display` | `src/ast/types.rs:268` | Type formatting (no manual format strings) |
| `ModuleSystem` | `src/module_system/mod.rs` | Module resolution (no path guessing) |

### Tier 2: AST Already Parsed

The LSP already parses every file via `Parser::parse_program()`. The AST is available.
There is ZERO reason to re-parse source text with string operations.

## Refactor Tasks

### Task 1: hover/ subsystem (9 files, 2101 lines)

**Files to change:**
- `hover/expressions.rs` (177 lines) — DELETE most of this. The TypeContext already has
  inferred types for all expressions. This file reimplements the typechecker poorly.
- `hover/format_string.rs` (243 lines) — Lines 104-184 are fallback string parsing that
  duplicates the parser. Delete them; the parser already succeeded on line 83-87.
- `hover/builtins.rs` (251 lines) — Replace hardcoded builtin docs with queries to
  `intrinsics::get_all_intrinsics()` which has doc strings and categories.
- `hover/structs.rs` (66 lines) — Use `TypeContext.get_struct_fields()` instead of
  text-based struct definition lookup.
- `hover/mod.rs` (656 lines) — Reduce fallback heuristic paths. When TypeContext is
  available (it almost always is), skip all heuristics.
- `hover/patterns.rs` (279 lines) — Use TypeContext for pattern type info.

**Key principle:** TypeContext is available from background analysis. Trust it. Delete
the heuristic fallbacks that do string parsing.

### Task 2: completion/ subsystem (4 files + semantic_completion.rs, ~1400 lines)

**Files to change:**
- `completion/context.rs` (570 lines) — Lines 296-308 use `.split(',')` to detect struct
  literal fields. Use AST `StructLiteral` node instead. Lines 200+ detect pattern match
  context via string ops — use AST `QuestionMatch` node.
- `completion/modules.rs` (54 lines) — Hardcoded `@std` module paths. Use ModuleSystem's
  package map to enumerate available modules.
- `completion/methods.rs` (46 lines) — Use TypeStore's `find_method_for_type()` and
  `get_all_functions()` for UFC method lookup.
- `semantic_completion.rs` (349 lines) — Already good (uses TypeContext). Clean up
  any remaining string ops.

### Task 3: navigation/ subsystem (7 files, 1548 lines)

**Files to change:**
- `navigation/imports.rs` (60 lines) — Line 12 has `find_import_info_from_ast()` but
  lines 30-60 reimplement import parsing with strings. Delete the string version.
- `navigation/definition.rs` (528 lines) — 40+ lines of path string matching
  (`.contains("stdlib")`, `.rfind("/std/")`) to detect stdlib files. Use
  `StdlibTypeRegistry.get_struct_source()` or `ModuleSystem` instead.
- `navigation/struct_fields.rs` (258 lines) — Lines 205-240 use `.starts_with()` patterns
  to find struct definitions. Use AST `Declaration::Struct` from parsed program.
- `navigation/utils.rs` (303 lines) — `find_symbol_definition_in_content()` does text
  search for definitions. Use TypeContext's `definition_locations` map.
- `navigation/references.rs` (272 lines) — Text-based reference search. Could use AST
  walking + TypeContext for accurate results.

### Task 4: document_store/ subsystem (9 files, 1147 lines)

**Files to change:**
- `document_store/builtin_registration.rs` (97 lines) — Manual primitive type registration.
  Use `primitives` module + `WellKnownTypes` + `intrinsics` registry instead.
- `document_store/reference_tracking.rs` (101 lines) — Reference kind detection uses
  string ops (`starts_with("+=")` etc). Use AST `Statement::VariableAssignment` nodes.
- `document_store/variable_extraction.rs` (102 lines) — Manual variable extraction from
  function bodies. TypeContext already has `variables` map with types.
- `document_store/symbol_search.rs` (153 lines) — Text-based symbol search across files.
  Could use TypeContext's `definition_locations` for indexed lookup.

### Task 5: code_action/ + inlay_hints (7 files, 1868 lines)

**Files to change:**
- `code_action/quick_fixes.rs` (290 lines) — CRITICAL: Lines 113, 119 parse diagnostic
  messages via `.contains("expected String")`. Use `CompileError` enum variants/error codes
  instead of matching on English text.
- `code_action/imports.rs` (273 lines) — Import resolution via string manipulation.
  Use `ModuleSystem` and AST `Declaration::ModuleImport`.
- `inlay_hints.rs` (604 lines) — Has good TypeContext path but fallback path (lines 200+)
  reimplements type inference with strings. Remove or simplify fallback.

### Task 6: Core cleanup (server.rs, utils.rs, stdlib_resolver.rs)

**Files to change:**
- `utils.rs` (705 lines) — `format_type()` should delegate to `AstType::Display` impl.
  Remove any manual type formatting.
- `stdlib_resolver.rs` (224 lines) — Hardcoded relative path guessing
  (`"./stdlib"`, `"../stdlib"`, `"../../stdlib"`). Use `BuildConfig::discover()` and
  `ModuleSystem` search paths.
- `server.rs` (1075 lines) — Ensure TypeContext is always propagated to all feature
  handlers. Currently some handlers don't receive it.

## Guiding Principles

1. **If the parser already parsed it, use the AST** — never re-parse source text
2. **If the typechecker already inferred it, use TypeContext** — never re-infer types
3. **If there's a name_utils function, use it** — never ad-hoc string split
4. **If there's a registry (intrinsics, well_known, stdlib_types), query it** — never hardcode
5. **Delete fallback heuristics** that duplicate compiler work — they're always wrong in edge cases
6. **Use error codes, not message text** — for diagnostic-based code actions
