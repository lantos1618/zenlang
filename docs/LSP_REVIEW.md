# Zen LSP -- Comprehensive File-by-File Audit

**Date:** 2026-02-08
**Scope:** Every file in `src/lsp/` (59 Rust source files, 13,181 lines)
**Methodology:** Every file read in full; each rated on a four-tier scale:

| Rating | Meaning |
|--------|---------|
| CLEAN  | No significant issues; uses compiler APIs correctly |
| OK     | Minor issues or minor hardcoded values; generally sound |
| WEAK   | Structural problems, significant text-based fallbacks, or code duplication |
| SLOP   | Hardcoded strings that belong in a registry, stub implementations, or dead/unreachable code |

---

## 1. Architecture Overview

```
                        +-----------------------+
                        |     LSP Client        |
                        |  (VS Code / editor)   |
                        +-----------+-----------+
                                    |
                            JSON-RPC / stdio
                                    |
                        +-----------v-----------+
                        |    server.rs           |
                        | ZenLanguageServer      |
                        | - Connection loop      |
                        | - Request dispatch     |
                        | - Notification dispatch|
                        | - Background analysis  |
                        |   worker thread        |
                        +-----------+-----------+
                                    |
              +---------------------+---------------------+
              |                     |                     |
     +--------v-------+   +--------v--------+   +--------v--------+
     |   Handlers     |   | DocumentStore   |   |  Utilities      |
     | (one per LSP   |   | (shared state)  |   |                 |
     |  method)       |   | Arc<RwLock<DS>> |   | helpers.rs      |
     +--------+-------+   +--------+--------+   | utils.rs        |
              |                     |            | types.rs        |
              |                     |            +-----------------+
              |                     |
   +----------+----------+         |
   |                     |         |
   v                     v         v
+--+---+  +------+  +---+------+---+------+
|hover/|  |nav/  |  |completion/| code_   |
|      |  |      |  |           | action/ |
+------+  +------+  +-----------+--------+
                                    |
                     +--------------+--------------+
                     |              |              |
              +------v------+ +----v----+  +------v------+
              | TypeContext  | | Parser  |  | Lexer       |
              | (typechecker)| | (AST)   |  | (tokens)    |
              +-------------+ +---------+  +-------------+
                  ^                ^               ^
                  |                |               |
             Compiler APIs used by LSP handlers
```

### Data Flow: LSP Request to Response

1. **Client sends JSON-RPC request** via stdio.
2. **`server.rs` dispatch loop** matches request method string to handler function.
3. **Handler** acquires `RwLock` read guard on `DocumentStore`.
4. **Handler** looks up the `Document` by URI, accesses `doc.ast`, `doc.type_context`, `doc.symbols`.
5. **Handler** queries `TypeContext` (authoritative semantic data from typechecker) for type info, or falls back to `SymbolInfo` maps and text-based heuristics.
6. **Handler** builds LSP response types and returns `Response`.
7. **`server.rs`** serializes and sends the response.

### Background Analysis Pipeline

1. `server.rs` receives `textDocument/didChange` or `textDocument/didOpen`.
2. `DocumentStore::update_document` debounces via content hashing (FNV-1a).
3. If content changed, an `AnalysisJob` is sent via `mpsc::channel` to the background worker.
4. Worker thread calls `analyzer::analyze_document` which runs full compiler pipeline (parse -> typecheck -> extract TypeContext).
5. `AnalysisResult` (containing `TypeContext`, diagnostics, updated AST) is sent back via another channel.
6. Main loop applies results to the `DocumentStore`, publishes diagnostics.

### Key Compiler APIs Used

| Compiler API | Used By | Purpose |
|---|---|---|
| `crate::parser::Parser` | analyzer, signature_help, format_string, context, semantic_completion | Parse source to AST |
| `crate::lexer::Lexer` | semantic_tokens, rename, signature_help | Tokenize source |
| `crate::typechecker` | analyzer | Full typecheck for TypeContext |
| `TypeContext` (variables, functions, methods, structs, enums) | inlay_hints, hover, signature_help, type_query, pattern_checking | Authoritative type data |
| `crate::ast::Declaration` / `Statement` / `Expression` | Many | AST node matching |
| `crate::ast::primitives` | completion, semantic_tokens, builtins | Canonical type/keyword lists |
| `crate::well_known::well_known()` | completion, builtins, hover | Option/Result/Some/None/Ok/Err names |
| `crate::lsp::stdlib_resolver::StdlibResolver` | completion, code_action/imports | `@std` path resolution |
| `crate::formatting` | formatting.rs | Code formatting |

---

## 2. File-by-File Audit

### 2.1 Top-Level Files (20 files, 6,249 lines)

---

#### `src/lsp/mod.rs` -- 66 lines -- CLEAN

**Purpose:** Module root. Declares all submodules, defines `search_limits` constants (MAX_DIRECTORY_DEPTH=10, MAX_FILES_TO_PARSE=200, MAX_LINES_TO_SEARCH=10000), re-exports `ZenLanguageServer`.

**Compiler APIs:** None directly.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/server.rs` -- 1,066 lines -- OK

**Purpose:** Core LSP server. Contains `ZenLanguageServer` struct, `run()` connection loop, request/notification dispatch, background analysis worker thread, `position_to_byte_offset` for UTF-16-to-byte conversion, `apply_text_edit` for incremental sync, `compile_error_to_diagnostic`, `extract_error_info`, `enhance_error_message`, `generate_type_mismatch_hint`.

**Compiler APIs:** `crate::typechecker`, `crate::parser`, `crate::ast`, TypeContext, `crate::formatting`.

**Issues:**
1. `compile_error_to_diagnostic` is duplicated in `utils.rs` (identical logic).
2. `extract_error_info` and `enhance_error_message` use regex-like string parsing on compiler error messages instead of structured error types.
3. `generate_type_mismatch_hint` constructs diagnostic hints by parsing the text of error messages for "expected" and "found" types.
4. Large file -- dispatch logic, analysis worker, and error processing are all in one file.

**Recommendations:**
- Extract `compile_error_to_diagnostic` to a single location (it already exists in `utils.rs`).
- If the compiler's error types gain structured fields (expected_type, found_type), use them directly instead of string parsing.
- Consider splitting the background analysis worker into its own module.

---

#### `src/lsp/types.rs` -- 118 lines -- CLEAN

**Purpose:** Core data structures: `Document`, `SymbolInfo`, `UfcMethodInfo`, `ZenCompletionContext` (enum: UfcMethod/ModulePath/StructLiteral/PatternMatch/General), `SymbolScope`, `AnalysisJob`, `AnalysisResult`, `hash_content` (FNV-1a).

**Compiler APIs:** `AstType`, `Declaration`, `TypeContext`, `Span`.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/utils.rs` -- 645 lines -- OK

**Purpose:** Utility functions with comprehensive test suite. `tokenize_with_lines` (deprecated, kept for tests), `byte_offset_to_lsp_position`, `span_to_lsp_range`, `compile_error_to_diagnostic` (duplicated from server.rs), `format_type` (AstType to display string), `format_symbol_kind`, `symbol_kind_to_completion_kind`. 185 lines of tests.

**Compiler APIs:** `Lexer`, `AstType`, `Span`, `CompileError`.

**Issues:**
1. `compile_error_to_diagnostic` is duplicated from `server.rs`.
2. `tokenize_with_lines` is deprecated but retained for test coverage.

**Recommendations:**
- Remove duplicate `compile_error_to_diagnostic` from `server.rs`, keep only in `utils.rs`.
- If `tokenize_with_lines` tests are valuable, refactor them to test the non-deprecated path.

---

#### `src/lsp/helpers.rs` -- 321 lines -- CLEAN

**Purpose:** Response construction helpers and handler boilerplate reduction. `HasDocumentUri` trait with 13 implementations for LSP param types, `with_document` helper, `null_response`, `success_response`, `success_response_id`, `error_response_id`, `try_parse_params`, `try_read`, `try_write`, `parse_params!` and `lock_store!` macros, `zen_code_block`, `type_context_to_lsp_location`, `char_pos_to_byte_pos`.

**Compiler APIs:** `TypeContext` (for location lookup).

**Issues:** None. Well-factored helper layer.

**Recommendations:** None.

---

#### `src/lsp/analyzer.rs` -- 238 lines -- CLEAN

**Purpose:** Document analysis pipeline. `analyze_document` runs parse + typecheck + extracts TypeContext. `run_compiler_analysis_with_context` drives the full compiler pipeline. `load_imports_for_program` resolves import files. `check_allocator_usage` emits custom diagnostics. Delegates pattern exhaustiveness checking to `pattern_checking.rs`.

**Compiler APIs:** `Parser`, `typechecker::check_program`, `TypeContext`, `Declaration`, `CompileError`.

**Issues:** None. Clean use of compiler APIs.

**Recommendations:** None.

---

#### `src/lsp/type_query.rs` -- 304 lines -- CLEAN

**Purpose:** `TypeQuery` struct wrapping optional `TypeContext` reference. Provides uniform query interface: `resolve_variable`, `resolve_function`, `resolve_method`, `resolve_struct_field`, `get_struct_fields`, `get_enum_variants`, `infer_literal_type`, `resolve_receiver_type`, `resolve_chain_type`, `infer_variable_type_unified`. Falls back to literal inference when TypeContext is unavailable.

**Compiler APIs:** `TypeContext`, `AstType`, `Expression`.

**Issues:** None. Well-designed abstraction layer.

**Recommendations:** None.

---

#### `src/lsp/semantic_completion.rs` -- 349 lines -- CLEAN

**Purpose:** Semantic dot-completions using TypeContext. `resolve_receiver_type` uses the parser to parse expressions and resolve types. `get_semantic_dot_completions` returns struct fields, methods, and UFC functions for a resolved type. Handles generic type specialization (e.g., `Vec<T>` -> concrete element type).

**Compiler APIs:** `Parser`, `Lexer`, `TypeContext`, `AstType`, `Expression`.

**Issues:** None. Good use of parser for expression analysis.

**Recommendations:** None.

---

#### `src/lsp/semantic_tokens.rs` -- 368 lines -- WEAK

**Purpose:** Semantic token generation for syntax highlighting. Uses the compiler Lexer for tokenization. `provide_semantic_tokens_full` generates tokens with custom comment extraction. `classify_identifier` distinguishes types from variables/functions. `classify_type` recognizes type names.

**Compiler APIs:** `Lexer`, `TokenKind`.

**Issues:**
1. `classify_type` has hardcoded strings: `"String"`, `"StaticString"`, `"Option"`, `"Result"`, `"Allocator"`, `"GPA"`, `"Error"`, `"HashMap"`, `"Vec"`, `"DynVec"`.
2. Does not use `primitives::PRIMITIVE_TYPE_MAP` or `well_known()` for type classification, even though these registries exist.
3. Comment extraction uses byte-level scanning with `content.find("//")` rather than leveraging lexer tokens.

**Recommendations:**
- Replace hardcoded type lists with `primitives::PRIMITIVE_TYPE_MAP` check + `well_known()` type names + `stdlib_types()` check.
- Use `looks_like_type_name()` from `crate::ast` for the PascalCase heuristic (already available).

---

#### `src/lsp/rename.rs` -- 603 lines -- WEAK

**Purpose:** Rename handling. Uses Lexer for identifier validation. `determine_symbol_scope` classifies symbols as Local/ModuleLevel. `rename_in_content` does text-based replacement with word boundary checking. `rename_in_file` reads files from disk and applies text replacement.

**Compiler APIs:** `Lexer`, `TokenKind`, `Declaration`.

**Issues:**
1. `determine_symbol_scope` is duplicated from `navigation/scope.rs` (identical logic).
2. `rename_in_content` is entirely text-based string replacement with manual word-boundary detection, not AST-aware.
3. `collect_workspace_files` duplicates directory traversal logic that exists in `indexing.rs`.

**Recommendations:**
- Remove duplicated `determine_symbol_scope`, import from `navigation/scope.rs`.
- Consider AST-based rename for local variables where the AST is available.
- Consolidate directory traversal into a shared utility.

---

#### `src/lsp/signature_help.rs` -- 335 lines -- CLEAN

**Purpose:** Signature help. `find_function_call_at_position` uses the Parser to extract the function name from the expression before the opening parenthesis. `find_function_in_type_context` queries TypeContext for authoritative parameter types. Falls back to `SymbolInfo` from doc/stdlib/workspace symbols.

**Compiler APIs:** `Parser`, `Lexer`, `TypeContext`, `Expression`, `AstType`.

**Issues:** None. Good use of parser for call-site analysis.

**Recommendations:** None.

---

#### `src/lsp/inlay_hints.rs` -- 231 lines -- CLEAN

**Purpose:** Inlay type hints for variable declarations. Uses TypeContext exclusively -- does not show hints when TypeContext is unavailable (deliberate design choice). Walks AST for `Statement::VariableDeclaration` with inferred types, looks up resolved types from `TypeContext.variables`.

**Compiler APIs:** `TypeContext`, `Declaration`, `Statement`, `Expression`, `VariableDeclarationType`, `AstType`.

**Issues:** None. Exemplary use of TypeContext.

**Recommendations:** None.

---

#### `src/lsp/call_hierarchy.rs` -- 390 lines -- WEAK

**Purpose:** Call hierarchy (prepare/incoming/outgoing). Implements `textDocument/prepareCallHierarchy`, `callHierarchy/incomingCalls`, `callHierarchy/outgoingCalls`.

**Compiler APIs:** `Declaration` (for AST symbol matching).

**Issues:**
1. `find_function_references` uses text-based string search (`line.find(func_name)`) across all documents to find callers.
2. `find_outgoing_calls_in_function` uses text-based scanning to find function calls within a function body by looking for `name(` patterns.
3. `find_function_range_from_content` uses brace-counting to find function body boundaries.
4. No use of TypeContext, AST expression walking, or reference tracking.

**Recommendations:**
- Use AST expression walking to find FunctionCall/MethodCall expressions for outgoing calls.
- Use the reference tracking infrastructure (when completed) for incoming calls.
- Use `find_function_range` from `navigation/utils.rs` instead of reimplementing brace counting.

---

#### `src/lsp/code_lens.rs` -- 177 lines -- OK

**Purpose:** Code lenses providing Run/Build/Test buttons above functions. Uses AST `Declaration::Function` with spans for position detection, falls back to text search for `main =` pattern.

**Compiler APIs:** `Declaration`, `Span`.

**Issues:**
1. Text-based fallback for `main =` pattern when AST spans are missing.
2. Minor: hardcoded `"main"` check could use a constant.

**Recommendations:**
- Minor: ensure AST spans are always populated to eliminate text fallback.

---

#### `src/lsp/symbols.rs` -- 175 lines -- OK

**Purpose:** Document symbols and workspace symbol search handlers. `handle_document_symbol` converts `doc.symbols` to LSP `DocumentSymbol` format. `handle_workspace_symbol` searches across all documents plus stdlib/workspace symbols.

**Compiler APIs:** None directly (consumes `SymbolInfo` from DocumentStore).

**Issues:**
1. `handle_workspace_symbol` performs linear scan of all documents, stdlib_symbols, and workspace_symbols for every query keystroke.

**Recommendations:**
- Consider indexing symbols by prefix trie for faster workspace symbol search.

---

#### `src/lsp/symbol_extraction.rs` -- 415 lines -- WEAK

**Purpose:** Two symbol extraction functions for different contexts. `extract_symbols_static` extracts symbols without position tracking (used for indexing). `extract_symbols_with_path` extracts symbols with source location tracking and structured params.

**Compiler APIs:** `Declaration`, `AstType`.

**Issues:**
1. Massive code duplication between `extract_symbols_static` and `extract_symbols_with_path` -- the two functions walk the same AST declarations and differ only in whether they track spans and structured params.
2. Position finding in `extract_symbols_with_path` uses text search (`find_declaration_line`) as a fallback alongside span-based positioning.

**Recommendations:**
- Merge into a single extraction function with an options parameter controlling whether to include position/param data.
- Remove text-based position finding if AST spans are reliable.

---

#### `src/lsp/formatting.rs` -- 63 lines -- CLEAN

**Purpose:** Thin wrapper delegating to `crate::formatting::format_zen_code`. Handles the `textDocument/formatting` request.

**Compiler APIs:** `crate::formatting::format_zen_code`.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/pattern_checking.rs` -- 65 lines -- CLEAN

**Purpose:** Pattern exhaustiveness checking. Builds an enum registry from TypeContext and delegates to `crate::typechecker::validate_match_exhaustiveness`.

**Compiler APIs:** `TypeContext`, `crate::typechecker::validate_match_exhaustiveness`.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/indexing.rs` -- 96 lines -- OK

**Purpose:** Workspace file discovery and stdlib path resolution. `index_workspace_files` finds all `.zen` files in the workspace. `index_stdlib` finds stdlib files. `find_stdlib_path` discovers the stdlib directory.

**Compiler APIs:** None (filesystem operations).

**Issues:**
1. Directory traversal logic overlaps with `rename.rs::collect_workspace_files` and `document_store/symbol_search.rs::search_directory_for_symbol_bounded`.

**Recommendations:**
- Consolidate all directory traversal into this module as the single entry point.

---

#### `src/lsp/stdlib_resolver.rs` -- 224 lines -- CLEAN

**Purpose:** `StdlibResolver` struct providing `@std` module path resolution. `resolve_path` converts `@std.io.files` to filesystem path. `list_modules` lists available stdlib modules. `path_to_module_path` converts filesystem paths back to `@std.x.y` format. Caches discovered stdlib path.

**Compiler APIs:** None (filesystem + path manipulation).

**Issues:** None.

**Recommendations:** None.

---

### 2.2 hover/ Subdirectory (9 files, 1,671 lines)

---

#### `src/lsp/hover/mod.rs` -- 654 lines -- OK

**Purpose:** Main hover handler. Priority-ordered check chain: format string fields -> method calls -> enum variant access -> struct constructor -> pattern match context -> document symbols -> TypeContext -> stdlib symbols -> workspace documents -> import paths -> builtins -> definition location lookup. Uses `or_else` chaining to try each strategy.

**Compiler APIs:** `TypeContext`, `Declaration`, `Expression`, `AstType`, `Span`.

**Issues:**
1. Complex priority chain is hard to follow -- 15+ resolution strategies in sequence.
2. Some strategies use text-based content scanning alongside AST queries.

**Recommendations:**
- Consider documenting the priority chain more explicitly or adding a table comment.
- Generally well-structured despite complexity.

---

#### `src/lsp/hover/response.rs` -- 140 lines -- CLEAN

**Purpose:** Hover response formatting. `build_hover_response` and `build_hover_response_with_docs` construct `Hover` objects with markdown code blocks. `format_params_documentation` formats structured parameter documentation.

**Compiler APIs:** `AstType`.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/hover/builtins.rs` -- 169 lines -- OK

**Purpose:** Built-in hover text. `get_builtin_hover` checks intrinsic functions, well-known types, stdlib types, and primitive types. Provides markdown hover documentation for each.

**Compiler APIs:** `crate::codegen::intrinsics::INTRINSIC_REGISTRY`, `well_known()`, `stdlib_types()`, `primitives::PRIMITIVE_TYPE_MAP`.

**Issues:**
1. Hardcoded keyword list for `"comptime"`, `"defer"`, `"loop"`, etc. -- partially overlaps with `primitives::CONTROL_FLOW`.

**Recommendations:**
- Use `primitives::CONTROL_FLOW` for the keyword hover checks.

---

#### `src/lsp/hover/expressions.rs` -- 211 lines -- OK

**Purpose:** Expression hover analysis. `try_member_or_method_hover` handles MemberAccess and MethodCall expressions. Resolves receiver type via TypeQuery, then looks up field/method types from TypeContext.

**Compiler APIs:** `TypeQuery`, `TypeContext`, `Declaration`, `Expression`, `AstType`.

**Issues:**
1. Falls back to text scanning when AST expression matching fails.

**Recommendations:** None major.

---

#### `src/lsp/hover/format_string.rs` -- 118 lines -- CLEAN

**Purpose:** Hover for format string interpolations like `${expr}`. Uses the Parser to parse the expression inside `${}` and resolves its type.

**Compiler APIs:** `Parser`, `Lexer`, `Expression`, `TypeQuery`.

**Issues:** None. Good use of parser.

**Recommendations:** None.

---

#### `src/lsp/hover/imports.rs` -- 4 lines -- CLEAN

**Purpose:** Re-exports `find_import_at_position` from `navigation/imports.rs`.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/hover/inference.rs` -- 53 lines -- CLEAN

**Purpose:** Variable type inference for hover. `infer_variable_type_from_context` uses `TypeQuery` for unified type inference.

**Compiler APIs:** `TypeQuery`, `TypeContext`, `AstType`, `Expression`.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/hover/patterns.rs` -- 277 lines -- OK

**Purpose:** Pattern match hover. Extracts the scrutinee expression from switch/match statements, resolves its type via compiler SEMA (TypeContext function return types and variable types), and provides variant hover info.

**Compiler APIs:** `TypeContext`, `Declaration`, `AstType`, `Expression`, `Statement`.

**Issues:**
1. `extract_matched_type_from_source` does text-based scanning to find the switch expression when AST span matching fails.
2. `find_function_containing_switch` uses line-based text scanning.

**Recommendations:**
- Use AST statement walking to find the enclosing switch instead of text scanning.

---

#### `src/lsp/hover/structs.rs` -- 45 lines -- CLEAN

**Purpose:** Struct definition formatting and variable hover helpers. `format_struct_definition` creates hover markdown from Declaration::Struct. `try_variable_hover` looks up variable types from TypeContext.

**Compiler APIs:** `Declaration`, `TypeContext`.

**Issues:** None.

**Recommendations:** None.

---

### 2.3 navigation/ Subdirectory (9 files, 1,766 lines)

---

#### `src/lsp/navigation/mod.rs` -- 20 lines -- CLEAN

**Purpose:** Module declarations and re-exports for navigation handlers.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/navigation/definition.rs` -- 491 lines -- OK

**Purpose:** Go-to-definition. 10+ resolution strategies chained with `or_else`: struct field definition, import path resolution, TypeContext-based lookup, document symbol lookup, stdlib symbol lookup, workspace search, UFC method resolution, struct field resolution, enum variant resolution.

**Compiler APIs:** `TypeContext`, `Declaration`, `AstType`, `StdlibResolver`.

**Issues:**
1. Complex resolution chain is hard to follow (similar to hover/mod.rs).
2. `find_definition_location` in utils.rs uses text-based content scanning.

**Recommendations:**
- Generally well-structured; complexity is inherent to go-to-definition.

---

#### `src/lsp/navigation/references.rs` -- 272 lines -- OK

**Purpose:** Find references. Scope-aware search using `determine_symbol_scope`. Searches current document and optionally workspace files. Classifies reference kinds (Read/Write) based on surrounding syntax.

**Compiler APIs:** `Declaration`.

**Issues:**
1. Reference finding uses text-based scanning (`find_all_symbol_occurrences`) rather than AST-based reference tracking.

**Recommendations:**
- When reference tracking is implemented in DocumentStore, switch to AST-based references.

---

#### `src/lsp/navigation/highlight.rs` -- 39 lines -- CLEAN

**Purpose:** Document highlight. Delegates to `find_all_symbol_occurrences` from utils.rs.

**Compiler APIs:** None directly.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/navigation/type_definition.rs` -- 70 lines -- OK

**Purpose:** Go-to-type-definition. Looks up the symbol, extracts type name from `SymbolInfo.type_info`, then finds the type definition.

**Compiler APIs:** `AstType`.

**Issues:**
1. Type name extraction from `AstType` handles only a subset of type variants.

**Recommendations:** None major.

---

#### `src/lsp/navigation/utils.rs` -- 303 lines -- WEAK

**Purpose:** Shared navigation utilities. `find_symbol_at_position` extracts the word at cursor using character-level boundary detection. `find_function_range` uses brace-counting to find function body boundaries. `find_all_symbol_occurrences` finds symbol occurrences via text search. `find_stdlib_location` resolves stdlib symbols to locations. `is_in_string_or_comment` checks if a position is inside a string literal or comment.

**Compiler APIs:** None directly.

**Issues:**
1. `find_symbol_at_position` uses character-level scanning rather than lexer tokens.
2. `find_function_range` uses brace-counting instead of AST function spans.
3. `find_all_symbol_occurrences` uses text search with manual word-boundary detection.
4. `is_in_string_or_comment` is a basic heuristic that may fail for multi-line strings or nested quotes.

**Recommendations:**
- Use Lexer to tokenize and find the token at position for `find_symbol_at_position`.
- Use AST function Declaration spans for `find_function_range`.
- Use AST expression/statement walking for symbol occurrence finding.

---

#### `src/lsp/navigation/imports.rs` -- 26 lines -- CLEAN

**Purpose:** `ImportInfo` struct and `find_import_at_position` using AST `Declaration::ModuleImport` matching.

**Compiler APIs:** `Declaration`, `Span`.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/navigation/scope.rs` -- 64 lines -- OK

**Purpose:** `determine_symbol_scope` -- classifies a symbol as Local, ModuleLevel, or Unknown by checking if it appears in AST function bodies vs top-level declarations.

**Compiler APIs:** `Declaration`, `Statement`.

**Issues:**
1. This logic is duplicated in `rename.rs`.

**Recommendations:**
- Ensure `rename.rs` imports from here instead of duplicating.

---

#### `src/lsp/navigation/struct_fields.rs` -- 258 lines -- OK

**Purpose:** Struct field go-to-definition. Resolves the receiver type of a member access expression, finds the struct definition, and locates the specific field.

**Compiler APIs:** `TypeContext`, `Declaration`, `AstType`, `Expression`.

**Issues:**
1. `infer_receiver_type_from_content` has a text-based fallback path when TypeContext resolution fails.

**Recommendations:**
- Minor: The text-based fallback is reasonable as a last resort.

---

#### `src/lsp/navigation/ufc.rs` -- 223 lines -- SLOP

**Purpose:** UFC (Uniform Function Call) method resolution. Finds functions whose first parameter type matches the receiver type, enabling `receiver.method()` calls.

**Compiler APIs:** `Declaration`, `AstType`.

**Issues:**
1. Hardcoded `"GPA"` as alias for `"GeneralPurposeAllocator"` (line ~90).
2. Hardcoded special-case for `"loop"` and `"iter"` method names (line ~100).
3. `type_matches` function has hardcoded pointer-type strings like `"*u8"`, `"*const u8"`.
4. `find_symbol_definition_type` uses text-based content scanning.

**Recommendations:**
- Replace `"GPA"` hardcode with a type alias registry or TypeContext lookup.
- Remove `"loop"`/`"iter"` special cases if possible, or document why they are needed.
- Use `AstType` structural matching instead of string comparisons for pointer types.

---

### 2.4 completion/ Subdirectory (5 files, 1,078 lines)

---

#### `src/lsp/completion/mod.rs` -- 337 lines -- CLEAN

**Purpose:** Main completion handler. Context detection dispatches to UfcMethod, ModulePath, StructLiteral, PatternMatch, or General completions. General completions include keywords from `primitives`, well-known types from `well_known()`, document symbols, stdlib symbols with auto-import, and workspace symbols. Priority tiers ensure proper ordering.

**Compiler APIs:** `primitives::PRIMITIVE_TYPE_MAP`, `well_known()`.

**Issues:** None. Uses registries correctly.

**Recommendations:** None.

---

#### `src/lsp/completion/context.rs` -- 536 lines -- OK

**Purpose:** Completion context detection. `get_completion_context` analyzes text before cursor to determine context type. `get_struct_literal_completions` and `get_pattern_match_completions` provide context-specific completions using AST data.

**Compiler APIs:** `Declaration`, `AstType`, `Expression`, `Parser`, `Lexer`.

**Issues:**
1. Context detection in `get_completion_context` uses text analysis (scanning for `.`, `::`, `{`) rather than AST position mapping.
2. `infer_type_from_expression_text` uses parser but falls back to text heuristics.

**Recommendations:**
- Text-based context detection is reasonable for completion triggers (`.`, `::`), which often occur in incomplete code that doesn't parse.

---

#### `src/lsp/completion/methods.rs` -- 46 lines -- SLOP

**Purpose:** Struct field completions and UFC method completions.

**Compiler APIs:** `DocumentStore`.

**Issues:**
1. `get_ufc_method_completions` is a **stub** -- returns `Vec::new()` unconditionally with a comment "UFC method completions are now handled by semantic_completion.rs".
2. Dead code that should be removed or redirected.

**Recommendations:**
- Either remove `get_ufc_method_completions` entirely and update callers, or redirect to `semantic_completion.rs`.

---

#### `src/lsp/completion/modules.rs` -- 54 lines -- CLEAN

**Purpose:** Module path completions. Uses `StdlibResolver` to list available modules for `@std.` prefix completions.

**Compiler APIs:** `StdlibResolver`.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/completion/auto_import.rs` -- 105 lines -- CLEAN

**Purpose:** Auto-import completion. `create_completion_with_import` creates CompletionItem with additionalTextEdits to add import statements. Uses AST-based import position detection.

**Compiler APIs:** `Declaration`, `AstType`.

**Issues:** None.

**Recommendations:** None.

---

### 2.5 document_store/ Subdirectory (8 files, 1,129 lines)

---

#### `src/lsp/document_store/mod.rs` -- 200 lines -- OK

**Purpose:** `DocumentStore` struct holding documents (HashMap<Url, Document>), stdlib_symbols, workspace_symbols, stdlib_resolver, workspace_root. Provides `resolve_symbol` (searches doc -> workspace -> stdlib), `find_struct_definition`, workspace indexing.

**Compiler APIs:** `Declaration`.

**Issues:**
1. `find_struct_definition` iterates all documents linearly.

**Recommendations:**
- Consider a type-name index for faster struct definition lookup.

---

#### `src/lsp/document_store/document_lifecycle.rs` -- 137 lines -- CLEAN

**Purpose:** Document open/update/close lifecycle. `open_document` parses content, extracts symbols, registers document. `update_document` does content hashing for change detection, debounces re-parsing, dispatches background analysis. `close_document` removes document.

**Compiler APIs:** None directly (delegates to parsing.rs and symbol_extraction.rs).

**Issues:** None. Clean lifecycle management.

**Recommendations:** None.

---

#### `src/lsp/document_store/parsing.rs` -- 69 lines -- CLEAN

**Purpose:** Parse and tokenize document content. `parse` creates Parser from Lexer and returns AST. `tokenize` creates Lexer and collects tokens.

**Compiler APIs:** `Parser`, `Lexer`, `Token`, `Declaration`.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/document_store/builtin_registration.rs` -- 105 lines -- CLEAN

**Purpose:** Registers built-in types as symbols in DocumentStore. Registers primitive types from `PRIMITIVE_TYPE_MAP`, well-known types from `well_known()`, and compiler intrinsics from `INTRINSIC_REGISTRY`.

**Compiler APIs:** `primitives::PRIMITIVE_TYPE_MAP`, `well_known()`, `INTRINSIC_REGISTRY`.

**Issues:** None. Uses canonical registries.

**Recommendations:** None.

---

#### `src/lsp/document_store/reference_tracking.rs` -- 101 lines -- SLOP

**Purpose:** AST expression/statement reference tracking. `find_references_in_expression` and `find_references_in_statements` walk AST nodes.

**Issues:**
1. **Mostly stubs.** Comments throughout say `"// TODO: Add reference location"` but the actual tracking is not implemented -- the functions identify identifiers in expressions but do not record reference locations anywhere.
2. The `symbols` HashMap parameter is modified to set `definition_uri` but reference *locations* are never pushed to `references: Vec<Location>`.

**Recommendations:**
- Implement actual reference location recording, or remove the dead code.
- When implemented, this would enable AST-based find-references, replacing text-based scanning.

---

#### `src/lsp/document_store/symbol_extraction.rs` -- 184 lines -- OK

**Purpose:** Symbol extraction from AST within DocumentStore context. `extract_symbols` parses and delegates. `extract_symbols_from_ast` walks declarations (Function, Struct, Enum, Constant, TraitImplementation) and builds SymbolInfo entries.

**Compiler APIs:** `Declaration`, `AstType`.

**Issues:**
1. `find_impl_block_range` uses text search for `"Type.implements"` pattern instead of AST spans.
2. Overlaps with top-level `symbol_extraction.rs` -- two separate symbol extraction systems.

**Recommendations:**
- Consolidate with top-level `symbol_extraction.rs` to have a single symbol extraction path.
- Use AST spans for impl block ranges.

---

#### `src/lsp/document_store/symbol_search.rs` -- 153 lines -- OK

**Purpose:** Symbol position finding and workspace symbol search. `find_declaration_position` locates declaration positions via text search. `search_workspace_for_symbol` recursively searches `.zen` files with depth and file count bounds.

**Compiler APIs:** `Declaration`.

**Issues:**
1. `find_declaration_position` uses text-based search with `find_word_in_line_for_symbol`.
2. `find_word_in_line_for_symbol` uses character-level boundary detection.
3. Directory traversal overlaps with `indexing.rs` and `rename.rs`.

**Recommendations:**
- Use AST span data instead of text search for declaration positions.
- Consolidate directory traversal.

---

#### `src/lsp/document_store/utilities.rs` -- 78 lines -- CLEAN

**Purpose:** Helper functions for reducing duplication. `make_range`, `dummy_range`, `make_symbol`, `make_enum_symbol` -- factory functions for constructing `SymbolInfo` and `Range` objects.

**Issues:** None.

**Recommendations:** None.

---

#### `src/lsp/document_store/variable_extraction.rs` -- 102 lines -- OK

**Purpose:** Variable symbol extraction from function bodies. Walks `Statement::VariableDeclaration` nodes, infers types via `TypeQuery`, and registers variable symbols.

**Compiler APIs:** `Statement`, `Expression`, `AstType`, `TypeQuery`.

**Issues:**
1. `find_variable_position` uses text-based search for variable names instead of AST spans.

**Recommendations:**
- Use AST `Span` from `Statement::VariableDeclaration` when available.

---

### 2.6 code_action/ Subdirectory (6 files, 1,288 lines)

---

#### `src/lsp/code_action/mod.rs` -- 134 lines -- CLEAN

**Purpose:** Main code action handler. Dispatches based on diagnostic codes: `allocator-required` -> allocator fix, `type-mismatch` -> string conversion, `type-error` -> error handling, `undeclared-variable`/`undeclared-function` -> "did you mean" + missing import, `unused-variable` -> underscore prefix. Also provides extract variable/function refactorings for selections.

**Compiler APIs:** None directly (delegates to sub-modules).

**Issues:** None. Clean dispatch architecture.

**Recommendations:** None.

---

#### `src/lsp/code_action/imports.rs` -- 227 lines -- CLEAN

**Purpose:** Missing import quick-fix. Searches stdlib_symbols and workspace_symbols for undefined names. Creates import statement insertion edits. Uses AST for existing import detection (`is_symbol_imported`) and import position finding (`find_import_insert_position`).

**Compiler APIs:** `Declaration`, `StdlibResolver`.

**Issues:** None. Good use of AST for import analysis.

**Recommendations:** None.

---

#### `src/lsp/code_action/quick_fixes.rs` -- 323 lines -- OK

**Purpose:** Quick-fix code actions. `create_allocator_fix_action` adds `get_default_allocator()`. `create_string_conversion_action` offers String/StaticString conversions. `create_error_handling_action` suggests `.raise()` for `.unwrap()`. `create_unused_variable_fix` prefixes with underscore.

**Compiler APIs:** None (operates on diagnostic messages and text edits).

**Issues:**
1. `create_error_handling_action` uses text search (`line.find(".unwrap()")`) rather than AST expression matching.
2. `extract_expected_type` parses diagnostic message text to extract type names.
3. `utf16_offset_to_byte_offset` is duplicated in `refactorings.rs`.

**Recommendations:**
- Extract shared `utf16_offset_to_byte_offset` to a utility.
- String parsing of diagnostic messages is acceptable since diagnostics are text-based by nature.

---

#### `src/lsp/code_action/refactorings.rs` -- 397 lines -- OK

**Purpose:** Extract variable and extract function refactorings. `create_extract_variable_action` extracts selected expression into a variable. `create_extract_function_action` extracts selected code into a new function. Uses AST spans for function boundary detection.

**Compiler APIs:** `Declaration`, `Statement`, `Span`.

**Issues:**
1. `utf16_offset_to_byte_offset` is duplicated from `quick_fixes.rs`.
2. `generate_function_name` uses text heuristics.
3. Return type detection for extracted functions is hardcoded to `"void"`.

**Recommendations:**
- Extract shared `utf16_offset_to_byte_offset` to a utility.
- Use TypeContext to infer return types for extracted functions when available.

---

#### `src/lsp/code_action/suggestions.rs` -- 91 lines -- CLEAN

**Purpose:** "Did you mean" suggestions for undefined symbols. Uses Levenshtein distance to find similar symbol names across document, workspace, and stdlib symbols. Top 3 closest matches offered as quick-fix actions.

**Compiler APIs:** None (operates on symbol name maps).

**Issues:** None. Clean implementation.

**Recommendations:** None.

---

#### `src/lsp/code_action/utils.rs` -- 116 lines -- CLEAN

**Purpose:** Utility functions for code actions. `diagnostic_code` extracts error code from diagnostic. `extract_symbol_from_diagnostic` parses symbol names from diagnostic messages. `levenshtein_distance` for fuzzy matching. Includes unit tests.

**Issues:** None.

**Recommendations:** None.

---

## 3. Remaining Slop Summary

### Critical (SLOP-rated files)

| File | Lines | Issue | Fix Effort |
|------|-------|-------|------------|
| `navigation/ufc.rs` | 223 | Hardcoded "GPA", "loop", "iter", pointer type strings | Medium |
| `completion/methods.rs` | 46 | `get_ufc_method_completions` is a stub returning empty Vec | Low |
| `document_store/reference_tracking.rs` | 101 | All reference tracking is stubbed out (TODO comments) | High |

### Significant (WEAK-rated files)

| File | Lines | Issue | Fix Effort |
|------|-------|-------|------------|
| `semantic_tokens.rs` | 368 | 10+ hardcoded type strings instead of using registries | Low |
| `rename.rs` | 603 | Duplicated `determine_symbol_scope`, text-based rename | Medium |
| `call_hierarchy.rs` | 390 | Entirely text-based function reference/call finding | High |
| `symbol_extraction.rs` | 415 | Massive duplication between two extraction functions | Medium |
| `navigation/utils.rs` | 303 | Character-level word detection, brace-counting, text search | Medium |

### Code Duplication Inventory

| Duplicated Code | Location A | Location B | Lines Duplicated |
|---|---|---|---|
| `compile_error_to_diagnostic` | `server.rs` | `utils.rs` | ~40 |
| `determine_symbol_scope` | `rename.rs` | `navigation/scope.rs` | ~30 |
| `utf16_offset_to_byte_offset` | `code_action/quick_fixes.rs` | `code_action/refactorings.rs` | ~10 |
| `extract_symbols_*` | `symbol_extraction.rs` (two functions) | `document_store/symbol_extraction.rs` | ~200 |
| Directory traversal | `indexing.rs` | `rename.rs` | `document_store/symbol_search.rs` | ~30 each |

### Hardcoded Values That Should Use Registries

| Hardcode | File | Should Use |
|---|---|---|
| `"String"`, `"StaticString"`, `"Option"`, `"Result"`, `"Allocator"`, `"GPA"`, `"Error"`, `"HashMap"`, `"Vec"`, `"DynVec"` | `semantic_tokens.rs` | `primitives::PRIMITIVE_TYPE_MAP` + `well_known()` + `stdlib_types()` |
| `"GPA"` -> `"GeneralPurposeAllocator"` | `navigation/ufc.rs` | Type alias registry or TypeContext |
| `"loop"`, `"iter"` special cases | `navigation/ufc.rs` | Remove or document |
| `"*u8"`, `"*const u8"` | `navigation/ufc.rs` | `AstType` structural matching |
| `"comptime"`, `"defer"`, `"loop"` etc. | `hover/builtins.rs` | `primitives::CONTROL_FLOW` |

---

## 4. Quality Distribution

| Rating | Files | Lines | Percentage |
|--------|-------|-------|------------|
| CLEAN  | 33    | 5,044 | 38.3%      |
| OK     | 18    | 5,855 | 44.4%      |
| WEAK   | 5     | 2,079 | 15.8%      |
| SLOP   | 3     | 370   | 2.8%       |
| **Total** | **59** | **13,181** | **100%** |

---

## 5. Recommended Next Steps

### Priority 1: Quick Wins (Low effort, high impact)

1. **Remove stub `get_ufc_method_completions`** in `completion/methods.rs` and update callers to use `semantic_completion.rs` directly. (~30 min)

2. **Deduplicate `compile_error_to_diagnostic`** -- remove from `server.rs`, import from `utils.rs`. (~15 min)

3. **Deduplicate `determine_symbol_scope`** -- remove from `rename.rs`, import from `navigation/scope.rs`. (~15 min)

4. **Extract `utf16_offset_to_byte_offset`** from `code_action/quick_fixes.rs` and `code_action/refactorings.rs` into `code_action/utils.rs`. (~15 min)

5. **Replace hardcoded types in `semantic_tokens.rs`** with `primitives::PRIMITIVE_TYPE_MAP` + `well_known()` + `looks_like_type_name()`. (~1 hr)

### Priority 2: Structural Improvements (Medium effort)

6. **Merge `symbol_extraction.rs` dual functions** into a single extraction function with options. (~2 hrs)

7. **Replace hardcoded values in `navigation/ufc.rs`** with TypeContext/registry lookups. (~2 hrs)

8. **Consolidate directory traversal** across `indexing.rs`, `rename.rs`, and `document_store/symbol_search.rs` into a single shared utility. (~1 hr)

9. **Use Lexer for `find_symbol_at_position`** in `navigation/utils.rs` instead of character-level scanning. (~2 hrs)

### Priority 3: Architectural Improvements (High effort, high value)

10. **Implement reference tracking** in `document_store/reference_tracking.rs` -- currently all stubs. This would enable AST-based find-references, replacing text-based scanning in `navigation/references.rs`, `call_hierarchy.rs`, and `rename.rs`. (~1 week)

11. **Replace text-based call hierarchy** in `call_hierarchy.rs` with AST expression walking. (~1 day)

12. **AST-based rename** -- use AST scope analysis for local variable renames instead of text-based replacement. (~2 days)

### Priority 4: Future Enhancements

13. **Workspace symbol index** -- replace linear scanning in `symbols.rs` with a prefix trie or inverted index for sub-millisecond workspace symbol search.

14. **Incremental parsing** -- currently the full document is re-parsed on every change. Tree-sitter or incremental parser integration could reduce latency.

15. **Structured compiler errors** -- if the compiler gains structured error types with expected/found type fields, replace string parsing in `server.rs` (`extract_error_info`, `enhance_error_message`).
