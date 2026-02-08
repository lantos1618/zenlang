# Phase 4: LSP Improvements, Smarter Caching & Bug Fixes

## TL;DR

> **Quick Summary**: Fix 3 pre-existing bugs (completion tests API, enum instance methods, `!` operator parsing), add definition location tracking to TypeContext, replace text-based symbol resolution with position-based lookups, add smarter caching to the LSP analysis pipeline (reuse ModuleSystem, skip unchanged files), and remove redundant text fallback paths.
> 
> **Deliverables**:
> - Fix `!` operator parsing (lexer emits Symbol, parser expects Operator — universal bug)
> - Fix enum instance method resolution (generic name key mismatch in typechecker)
> - Fix lsp_completion_tests.rs API mismatch (10 calls passing wrong type)
> - Add `definition_locations` to TypeContext (position tracking for all symbol types)
> - Replace text-based symbol search in definition/hover with TypeContext lookups
> - Persist ModuleSystem across analysis runs with content-hash invalidation
> - Skip re-analysis for unchanged files (content_hash check)
> - Remove text fallback paths once position-based resolution is proven
> 
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: Task 1/2/3 (bug fixes) → Task 4 (TypeContext positions) → Task 5 (position-based resolution) → Task 7 (fallback removal)

---

## Context

### Original Request
Continue from 3-phase codebase dedup effort. Implement 6 improvements identified during Phase 3 research: TypeContext positions, replace text search, smarter caching, module caching, remove fallbacks, fix pre-existing bugs.

### Interview Summary
**Key Discussions**:
- Incremental parsing scope: user chose "smarter caching only" — reuse ModuleSystem, skip unchanged files, cache TypeContext per-module. NOT full incremental/tree-sitter parsing.
- Enum instance methods: user chose "fix the actual typechecker bug" — make enum methods work, not just #[ignore] the test.
- hashmap.zen `!entry.occupied`: Originally thought to be LSP-specific parse error. Research revealed it's a **universal parser bug** — lexer emits `Token::Symbol('!')` but parser expects `Token::Operator("!")`. They never match. hashmap.zen was never compiled (dead stdlib code).

**Research Findings**:
- TypeContext has 9 HashMaps, zero position data. Registration methods follow `register_X(&mut self, ...)` pattern.
- Definition resolution uses 10-step fallback chain ending with text search in `definition.rs:541-549`.
- `!` operator: lexer.rs:386 emits `Token::Symbol('!')`, parser operators.rs:117 matches `Token::Operator(op) if op == "!"` — token type mismatch.
- Enum method bug: `SafePtr<T>.is_valid` is registered with key `"SafePtr<T>.is_valid"` but looked up as `"SafePtr.is_valid"`. `find_ufc_method` does NOT strip generics from the registered key.
- ModuleSystem (module_system/mod.rs) has `HashMap<String, Program>` cache but analyzer.rs creates new instance per analysis run — cache is wasted.
- Document already has `content_hash: u64` (FNV-1a) for change detection within a file. Can extend to cross-analysis caching.
- lsp_analysis_tests.rs (Phase 3): 41 tests pass fine — no stale API issue (Metis confirmed).

### Metis Review
**Identified Gaps** (addressed):
- lsp_analysis_tests.rs stale fields was FALSE — tests pass. Removed from plan.
- hashmap.zen reframed from "LSP parse divergence" to "universal parser bug"
- ModuleSystem caching needs invalidation strategy (content-hash based)
- TypeContext flows through Monomorphizer and LLVMCompiler — new fields must be Default-able and Clone
- `!` operator fix must verify `!=` still works (lexer handles `!=` separately before Symbol, so safe)
- Fallback removal must be last, after position-based resolution is proven
- find_symbol_definition_in_content() used from 4+ locations, not just resolve_text_fallback

---

## Work Objectives

### Core Objective
Fix 3 pre-existing bugs, add position tracking to TypeContext, replace text-based symbol resolution, add smarter caching, and clean up fallback paths.

### Concrete Deliverables
- Parser handles `!` as unary prefix operator correctly
- Enum instance methods resolve through typechecker
- All test files compile and pass (completion, analysis, ptr_ref, navigation, behavioral, lib)
- TypeContext carries definition positions for functions, structs, enums, methods, variables, type_aliases
- Definition/hover resolution uses TypeContext positions instead of text search
- ModuleSystem persists across LSP analysis runs with cache invalidation
- Unchanged files skip re-analysis

### Definition of Done
- [ ] `cargo test --lib` → 143+ passed, 0 failed
- [ ] `cargo test --test ptr_ref_tests` → all passed including `test_enum_with_instance_methods`
- [ ] `cargo test --test lsp_completion_tests` → compiles and passes
- [ ] `cargo test --test lsp_navigation_tests` → all passed
- [ ] `cargo test --test lsp_analysis_tests` → 41+ passed
- [ ] `cargo test --test behavioral_tests` → all passed
- [ ] `!true`, `!x`, `!foo.bar` parse correctly
- [ ] `hashmap.zen` parses without SyntaxError

### Must Have
- `!` operator works universally (not just in LSP)
- Enum instance methods with generic types resolve correctly
- TypeContext definition_locations populated during typechecking
- ModuleSystem cache invalidation based on content hash
- All existing tests continue to pass

### Must NOT Have (Guardrails)
- NO full incremental/tree-sitter-style parsing — smarter caching only
- NO restructuring the 9-strategy method resolution chain — fix only the key mismatch
- NO proper `UnaryNot` AST node — keep `FunctionCall("not")` representation
- NO new test cases in lsp_completion_tests — only fix the API mismatch
- NO expression-level position tracking — only functions, structs, enums, methods, variables, type_aliases
- NO `Arc<Mutex<>>` on ModuleSystem — TypeChecker contains `RefCell<TypeStore>` (not Send/Sync)
- NO file-watcher or dependency graph for caching — just content-hash invalidation
- NO removing fallbacks before TypeContext positions are proven via tests
- NO changing TypeContext's `#[derive(Debug, Clone, Default)]` contract

---

## Verification Strategy (MANDATORY)

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
>
> ALL tasks in this plan MUST be verifiable WITHOUT any human action.

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: YES (tests-after — add targeted tests for new functionality)
- **Framework**: `cargo test` (Rust built-in test framework)

### Agent-Executed QA Scenarios (MANDATORY — ALL tasks)

All verification via `cargo test` commands and targeted Bash checks. No Playwright needed (compiler/LSP, not UI). Evidence captured via terminal output.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — independent bug fixes):
├── Task 1: Fix `!` operator parsing (lexer/parser token mismatch)
├── Task 2: Fix enum instance method resolution (generic key mismatch)
└── Task 3: Fix lsp_completion_tests.rs API mismatch

Wave 2 (After Wave 1 — architectural changes):
├── Task 4: Add definition_locations to TypeContext
└── Task 6: Smarter caching (persist ModuleSystem + skip unchanged files)

Wave 3 (After Wave 2 — depends on TypeContext positions):
├── Task 5: Replace text-based symbol search with TypeContext lookups
└── Task 7: Remove redundant text fallback paths
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 1 | None | None | 2, 3 |
| 2 | None | None | 1, 3 |
| 3 | None | None | 1, 2 |
| 4 | 1, 2, 3 (clean baseline) | 5, 7 | 6 |
| 5 | 4 | 7 | 6 |
| 6 | 1, 2, 3 (clean baseline) | None | 4, 5 |
| 7 | 5 | None | None (final) |

### Agent Dispatch Summary

| Wave | Tasks | Recommended Agents |
|------|-------|-------------------|
| 1 | 1, 2, 3 | 3× `task(category="quick", ...)` in parallel |
| 2 | 4, 6 | `task(category="unspecified-high", ...)` + `task(category="unspecified-low", ...)` in parallel |
| 3 | 5, 7 | `task(category="unspecified-high", ...)` → then `task(category="unspecified-low", ...)` sequential |

---

## TODOs

- [ ] 0. Establish test baseline

  **What to do**:
  - Run `cargo test --lib` to confirm 143 tests pass
  - Run `cargo test --test lsp_analysis_tests` to confirm 41 tests pass
  - Run `cargo test --test ptr_ref_tests` to see `test_enum_with_instance_methods` failure
  - Run `cargo test --test lsp_completion_tests 2>&1` to see 10 compilation errors
  - Record exact baseline counts for regression checking

  **Must NOT do**:
  - Change any code in this task

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  - **Reason**: Just running test commands, no code changes

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (must run before Wave 1)
  - **Blocks**: Tasks 1, 2, 3
  - **Blocked By**: None

  **References**:
  - `Cargo.toml` — test configuration
  - `tests/` directory — all test files

  **Acceptance Criteria**:
  - [ ] Baseline recorded — exact pass/fail counts for each test target
  - [ ] No code changes made

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Record test baseline
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1 | tail -5
      2. cargo test --test lsp_analysis_tests 2>&1 | tail -5
      3. cargo test --test ptr_ref_tests 2>&1 | tail -5
      4. cargo test --test lsp_completion_tests 2>&1 | tail -10
      5. cargo test --test lsp_navigation_tests 2>&1 | tail -5
      6. cargo test --test behavioral_tests 2>&1 | tail -5
    Expected Result: Exact baseline established
    Evidence: Terminal output captured
  ```

  **Commit**: NO

---

- [ ] 1. Fix `!` operator parsing (universal parser bug)

  **What to do**:
  - In `src/lexer.rs:380-387`: Change `Token::Symbol('!')` to `Token::Operator("!".to_string())` so the lexer emits the same token type the parser expects
  - Verify `!=` still works — the lexer checks `!=` first (line 382-384) before falling through to `!`, so this is safe
  - Add parser test for `!` expressions: `!true`, `!x`, `!foo.bar`, `!!x` — verify they parse to `FunctionCall { name: "not", args: [expr] }`
  - Add test that `hashmap.zen` parses without SyntaxError (the `!entry.occupied` pattern at line 145)
  - Run full test suite to verify no regressions

  **Must NOT do**:
  - Do NOT add a proper `UnaryNot` AST node — keep the `FunctionCall("not")` representation
  - Do NOT change how `!=` is lexed (it's handled separately and correctly)
  - Do NOT modify the parser's unary expression handling (operators.rs:117-125 is correct, only the lexer is wrong)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  - **Reason**: Single-line fix in lexer + test additions. Straightforward.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3)
  - **Blocks**: None directly (but Wave 2 needs clean baseline)
  - **Blocked By**: Task 0 (baseline)

  **References**:

  **Pattern References**:
  - `src/lexer.rs:380-387` — The bug: `Token::Symbol('!')` should be `Token::Operator("!".to_string())`
  - `src/lexer.rs:382-384` — `!=` handling (check first, safe): emits `Token::Operator("!=".to_string())`
  - `src/parser/expressions/operators.rs:105-135` — `parse_unary_expression` expects `Token::Operator(op) if op == "!"` at line 117

  **Test References**:
  - `src/parser/expressions/operators.rs:117-125` — The parser code that SHOULD match `!` but can't due to token type mismatch
  - `stdlib/collections/hashmap.zen:145` — `!entry.occupied` — real-world usage that fails

  **Acceptance Criteria**:
  - [ ] `cargo test --lib` → all pass (143+), 0 failed
  - [ ] New test: `!true` parses to `FunctionCall { name: "not", args: [Boolean(true)] }`
  - [ ] New test: `!foo.bar` parses to `FunctionCall { name: "not", args: [FieldAccess { ... }] }`
  - [ ] New test: `!=` still parses correctly as comparison operator
  - [ ] hashmap.zen parses without SyntaxError (test that parses the file content)

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: ! operator parses correctly
    Tool: Bash
    Steps:
      1. cargo test --lib -- test_not_operator 2>&1 | grep -E "passed|failed"
      2. Assert: tests pass
    Expected Result: New ! operator tests pass
    Evidence: Terminal output

  Scenario: hashmap.zen parses without error
    Tool: Bash
    Steps:
      1. cargo test --lib -- test_hashmap_parse 2>&1 | grep -E "passed|failed"
         (or: write a test that parses hashmap.zen content and asserts no error)
      2. Assert: parse succeeds
    Expected Result: No SyntaxError for !entry.occupied
    Evidence: Terminal output

  Scenario: != operator still works
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1 | grep "test result"
      2. Assert: all existing tests pass (no regression from != handling)
    Expected Result: 143+ tests pass
    Evidence: Terminal output

  Scenario: Full regression check
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1 | tail -3
      2. cargo test --test behavioral_tests 2>&1 | tail -3
      3. Assert: 0 failures in both
    Expected Result: No regressions
    Evidence: Terminal output
  ```

  **Commit**: YES (groups with Tasks 2, 3 — Wave 1)
  - Message: `fix(parser): emit ! as Operator token so unary not parses correctly`
  - Files: `src/lexer.rs`, test file(s)
  - Pre-commit: `cargo test --lib`

---

- [ ] 2. Fix UFC method key format — normalize generic type names at registration

  **What to do**:
  - **Root cause**: The `method_key` format `"{type_name}.{method}"` is fragile with generics. Functions like `SafePtr<T>.is_valid` are registered with key `"SafePtr<T>.is_valid"` but looked up as `"SafePtr.is_valid"`. This breaks ALL generic UFC methods, not just enums:
    - `SafePtr<T>.is_valid` → key `"SafePtr<T>.is_valid"`, lookup `"SafePtr.is_valid"` ❌
    - `HashMap<K, V>.get` → key `"HashMap<K, V>.get"`, lookup `"HashMap.get"` ❌
    - `Vec<T>.push` → key `"Vec<T>.push"`, lookup `"Vec.push"` ❌
  
  - **Fix: Normalize keys at registration time** in `src/name_utils.rs`:
    - Add a `normalize_ufc_name(func_name: &str) -> String` function that strips generic params from the type portion:
      - `"SafePtr<T>.is_valid"` → `"SafePtr.is_valid"`
      - `"HashMap<K, V>.get"` → `"HashMap.get"`
      - `"plain_function"` → `"plain_function"` (no change, no `.` separator)
    - Implementation: if the name contains `<` before `.`, strip from `<` to the matching `>` (handling nested generics like `Foo<Bar<T>>`)
  
  - **Apply normalization at registration** in `src/typechecker/declaration_checking.rs:22-27`:
    - Before calling `register_function(&func.name, signature)`, normalize: `register_function(&normalize_ufc_name(&func.name), signature)`
    - This way `find_ufc_method("SafePtr", "is_valid")` generates `"SafePtr.is_valid"` which now matches the stored key
  
  - **Preserve full generic name** in the `FunctionSignature` itself (params/return_type carry the generic info) — the key is just a lookup handle, not the source of truth for generic params

  - **Also update `build_type_context()`** in `src/typechecker/mod.rs` to normalize when transferring to TypeContext — same normalization for `methods` and `method_params` HashMaps

  - **Audit all `method_key` callers** to ensure consistent normalization:
    - `find_ufc_method` (mod.rs:170) — uses `method_key` for lookup ✓ (will match normalized key)
    - `try_resolve_static_call` (calls.rs:242) — uses `find_ufc_method` ✓ (same path)
    - `try_resolve_instance_method` (calls.rs:312) — uses `find_ufc_method` ✓ (same path)
    - TypeContext `methods`/`method_params` registration — normalize here too

  - **Add tests**:
    - Existing: `test_enum_with_instance_methods` should now pass
    - New: test that `Vec<T>.method_name` style UFC methods resolve correctly
    - New: test `normalize_ufc_name` directly with edge cases: no generics, single generic, nested generics, no dot (plain function)

  **Must NOT do**:
  - Do NOT restructure the 9-strategy method resolution chain
  - Do NOT change Strategy 1's `==` comparison on AstType (that's a separate structural equality issue)
  - Do NOT modify `extract_type_name` (it correctly handles Enum already)
  - Do NOT add new strategies — fix the key normalization only
  - Do NOT change the separator from `.` to something else (would require updating all callers)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  - **Reason**: Focused fix in name_utils + registration sites. Well-scoped normalization.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3)
  - **Blocks**: None directly (but Wave 2 needs clean baseline)
  - **Blocked By**: Task 0 (baseline)

  **References**:

  **Pattern References**:
  - `src/name_utils.rs:44-46` — `method_key`: `format!("{}.{}", type_name, method_name)` — where the fragile key is generated
  - `src/name_utils.rs:48-53` — `scoped_var_key` — example of a different key format using `::`
  - `src/typechecker/declaration_checking.rs:22-27` — Where functions are registered with `func.name` (includes `<T>`) — normalization goes here
  - `src/typechecker/mod.rs:170-173` — `find_ufc_method`: lookup side (generates `"TypeName.method"`)
  - `src/typechecker/mod.rs:280-430` — `build_type_context()`: also generates method keys for TypeContext — normalize here too

  **API/Type References**:
  - `src/typechecker/inference/calls.rs:285-315` — `try_resolve_instance_method` Strategy 1 (==), Strategy 2 (stdlib), Strategy 3 (find_ufc_method)
  - `src/typechecker/inference/calls.rs:240-244` — Static call path also uses `find_ufc_method`
  - `src/typechecker/inference/helpers.rs:7-14` — `extract_type_name`: correctly returns `"SafePtr"` for Enum types

  **Test References**:
  - `tests/ptr_ref_tests.rs:182-210` — `test_enum_with_instance_methods`: the failing test with `SafePtr<T>.is_valid`

  **Edge Cases to Test**:
  - `normalize_ufc_name("SafePtr<T>.is_valid")` → `"SafePtr.is_valid"` (single generic)
  - `normalize_ufc_name("HashMap<K, V>.get")` → `"HashMap.get"` (multiple generics)
  - `normalize_ufc_name("Foo<Bar<T>>.method")` → `"Foo.method"` (nested generics)
  - `normalize_ufc_name("plain_function")` → `"plain_function"` (no dot, no change)
  - `normalize_ufc_name("Point.distance")` → `"Point.distance"` (no generics, no change)

  **Acceptance Criteria**:
  - [ ] `cargo test --test ptr_ref_tests -- test_enum_with_instance_methods` → 1 passed, 0 failed
  - [ ] `cargo test --test ptr_ref_tests` → all passed, 0 failed
  - [ ] `cargo test --lib` → all pass (143+), 0 failed (no regression)
  - [ ] New unit test: `normalize_ufc_name` handles all edge cases (single/multiple/nested generics, no generics, no dot)
  - [ ] `cargo test --test behavioral_tests` → all pass (codegen not broken by key changes)

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Enum instance method resolves
    Tool: Bash
    Steps:
      1. cargo test --test ptr_ref_tests -- test_enum_with_instance_methods 2>&1
      2. Assert: output contains "1 passed; 0 failed"
    Expected Result: Test passes
    Evidence: Terminal output

  Scenario: normalize_ufc_name unit tests pass
    Tool: Bash
    Steps:
      1. cargo test --lib -- normalize_ufc_name 2>&1
      2. Assert: all pass
    Expected Result: Edge cases covered
    Evidence: Terminal output

  Scenario: No regression in method resolution or codegen
    Tool: Bash
    Steps:
      1. cargo test --test ptr_ref_tests 2>&1 | tail -3
      2. Assert: all tests pass, 0 failed
      3. cargo test --lib 2>&1 | tail -3
      4. Assert: 143+ passed, 0 failed
      5. cargo test --test behavioral_tests 2>&1 | tail -3
      6. Assert: all pass (key normalization doesn't break codegen)
    Expected Result: Zero regressions
    Evidence: Terminal output
  ```

  **Commit**: YES (groups with Tasks 1, 3 — Wave 1)
  - Message: `fix(typechecker): normalize generic type names in UFC method keys`
  - Files: `src/name_utils.rs`, `src/typechecker/declaration_checking.rs`, `src/typechecker/mod.rs`
  - Pre-commit: `cargo test --lib && cargo test --test ptr_ref_tests`

---

- [ ] 3. Fix lsp_completion_tests.rs API mismatch

  **What to do**:
  - `get_completion_context` in `src/lsp/completion/context.rs:15` expects `Option<&Document>` as 3rd parameter
  - All 10 test calls in `tests/lsp_completion_tests.rs` pass `&store` (a `&DocumentStore`)
  - Fix: Change all calls to pass `None` (tests don't need a Document for basic context detection)
  - Affected lines: 24, 40, 57, 71, 87, 103, 172, 190, 208, 224
  - If the test helper `create_test_store()` is no longer needed after fix, remove it or keep it (executor's discretion)

  **Must NOT do**:
  - Do NOT add new test cases — only fix the API mismatch
  - Do NOT change `get_completion_context`'s signature — it's correct
  - Do NOT refactor the test structure

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  - **Reason**: Trivial find-and-replace across 10 call sites.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2)
  - **Blocks**: None directly (but Wave 2 needs clean baseline)
  - **Blocked By**: Task 0 (baseline)

  **References**:

  **API/Type References**:
  - `src/lsp/completion/context.rs:15` — `get_completion_context(content: &str, position: Position, doc: Option<&Document>)` — the correct signature
  - `tests/lsp_completion_tests.rs:24,40,57,71,87,103,172,190,208,224` — All 10 call sites with wrong argument type

  **Acceptance Criteria**:
  - [ ] `cargo test --test lsp_completion_tests 2>&1 | grep "test result"` → compiles and all tests pass
  - [ ] `cargo test --lib` → 143+ passed (no regression)

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Completion tests compile and pass
    Tool: Bash
    Steps:
      1. cargo test --test lsp_completion_tests 2>&1 | tail -5
      2. Assert: "test result: ok" and 0 failed
    Expected Result: All completion tests pass
    Evidence: Terminal output

  Scenario: No regression
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1 | tail -3
      2. Assert: 143+ passed, 0 failed
    Expected Result: No regressions from test-only changes
    Evidence: Terminal output
  ```

  **Commit**: YES (groups with Tasks 1, 2 — Wave 1)
  - Message: `fix(tests): update lsp_completion_tests to use Option<&Document> API`
  - Files: `tests/lsp_completion_tests.rs`
  - Pre-commit: `cargo test --test lsp_completion_tests`

---

- [ ] 4. Add definition_locations to TypeContext

  **What to do**:
  - Add new HashMap fields to `TypeContext` (src/type_context.rs) for tracking definition positions:
    ```rust
    /// Definition locations: symbol_key -> (file_path, line, column)
    /// Uses same key format as existing HashMaps (e.g., "func_name", "TypeName.method", "scope::var")
    pub definition_locations: HashMap<String, DefinitionLocation>,
    ```
  - Define `DefinitionLocation` struct:
    ```rust
    #[derive(Debug, Clone, Default)]
    pub struct DefinitionLocation {
        pub file: Option<String>,  // file path (None for stdlib builtins)
        pub line: u32,
        pub column: u32,
        pub end_line: u32,
        pub end_column: u32,
    }
    ```
  - Add `register_location(&mut self, key: String, location: DefinitionLocation)` method following existing `register_X` pattern
  - Add convenience lookup: `get_location(&self, key: &str) -> Option<&DefinitionLocation>`
  - Populate locations during typechecking — in `src/typechecker/declaration_checking.rs`:
    - When registering functions (line 26): also register location from `func.span`
    - When registering structs (line 59): also register location from `struct_def.span`  
    - When registering enums: also register location from `enum_def.span`
  - Populate in `build_type_context()` (src/typechecker/mod.rs) when transferring data to TypeContext:
    - Functions, methods, variables — capture spans from typechecker state
  - Write tests verifying position data is populated correctly for each symbol type

  **Must NOT do**:
  - Do NOT change TypeContext's `#[derive(Debug, Clone, Default)]` — DefinitionLocation must also derive these
  - Do NOT track expression-level positions — only functions, structs, enums, methods, variables, type_aliases
  - Do NOT use `Url` or `lsp_types::Range` in TypeContext — it's compiler infrastructure, not LSP. Use primitive types (String, u32)
  - Do NOT add position tracking for symbols that don't have Span data in the AST
  - Do NOT break the existing Monomorphizer or LLVMCompiler consumption of TypeContext

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
  - **Reason**: Architectural change touching TypeContext (shared across compiler phases), typechecker registration, and build_type_context. Moderate complexity.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 6)
  - **Blocks**: Task 5, Task 7
  - **Blocked By**: Tasks 1, 2, 3 (need clean baseline)

  **References**:

  **Pattern References**:
  - `src/type_context.rs:22-59` — TypeContext struct with 9 existing HashMaps (follow this pattern for new field)
  - `src/type_context.rs:68-100` — `register_function`, `register_struct`, `register_enum` (follow this pattern)
  - `src/type_context.rs:61-66` — `FunctionType` struct (follow this pattern for DefinitionLocation)

  **API/Type References**:
  - `src/typechecker/declaration_checking.rs:10-60` — `collect_declaration_types` where functions/structs/enums are registered (add location registration here)
  - `src/typechecker/mod.rs:280-430` — `build_type_context()` where TypeChecker state transfers to TypeContext
  - `src/ast/declarations.rs` — Declaration types with Span fields (source of position data)
  - `src/lexer.rs` — `Span` struct definition (start, end, line, column)

  **Documentation References**:
  - TypeContext flows: `TypeChecker → Monomorphizer → LLVMCompiler` (also `Arc<TypeContext>` in LSP)
  - New fields must be `Default` (empty HashMap) so `TypeContext::new()` still works

  **Acceptance Criteria**:
  - [ ] TypeContext has `definition_locations: HashMap<String, DefinitionLocation>` field
  - [ ] DefinitionLocation derives Debug, Clone, Default
  - [ ] Functions registered with location during typechecking
  - [ ] Structs registered with location during typechecking
  - [ ] Enums registered with location during typechecking
  - [ ] Methods registered with location during typechecking
  - [ ] `cargo test --lib` → all pass, 0 failed
  - [ ] New tests verify position data is populated for each symbol type
  - [ ] `cargo test --test behavioral_tests` → all pass (codegen not broken)

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: TypeContext positions populated for functions
    Tool: Bash
    Steps:
      1. cargo test --lib -- test_type_context_function_location 2>&1
      2. Assert: test passes, location has correct line/column
    Expected Result: Function definition positions captured
    Evidence: Terminal output

  Scenario: TypeContext positions populated for structs and enums
    Tool: Bash
    Steps:
      1. cargo test --lib -- test_type_context_struct_location 2>&1
      2. cargo test --lib -- test_type_context_enum_location 2>&1
      3. Assert: both pass
    Expected Result: Struct/enum definition positions captured
    Evidence: Terminal output

  Scenario: No regression in codegen pipeline
    Tool: Bash
    Steps:
      1. cargo test --test behavioral_tests 2>&1 | tail -3
      2. Assert: all pass, 0 failed
    Expected Result: Monomorphizer/LLVMCompiler unaffected
    Evidence: Terminal output

  Scenario: Full test suite green
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1 | tail -3
      2. cargo test --test lsp_analysis_tests 2>&1 | tail -3
      3. cargo test --test ptr_ref_tests 2>&1 | tail -3
      4. Assert: all pass
    Expected Result: Zero regressions
    Evidence: Terminal output
  ```

  **Commit**: YES
  - Message: `feat(typechecker): add definition location tracking to TypeContext`
  - Files: `src/type_context.rs`, `src/typechecker/declaration_checking.rs`, `src/typechecker/mod.rs`, test file(s)
  - Pre-commit: `cargo test --lib && cargo test --test behavioral_tests`

---

- [ ] 5. Replace text-based symbol search with TypeContext lookups

  **What to do**:
  - In `src/lsp/navigation/definition.rs`: modify `resolve_symbol_definition()` to check TypeContext definition_locations EARLY in the chain (after qualified name resolution, before workspace search)
    - If `doc.type_context` has a location for the symbol, convert `DefinitionLocation` → `lsp_types::Location` and return immediately
    - This short-circuits the 10-step fallback chain for symbols with TypeContext positions
  - In `src/lsp/hover/mod.rs`: modify `handle_type_context_hover()` to use definition_locations for "go to definition" links in hover content
  - In `src/lsp/navigation/utils.rs`: mark `find_symbol_definition_in_content()` as `#[allow(dead_code)]` if no longer called from active paths (but keep the function — Task 7 removes it)
  - Create a helper function `type_context_to_lsp_location(loc: &DefinitionLocation) -> Option<lsp_types::Location>` in `src/lsp/helpers.rs` for the conversion
  - Test: existing `lsp_navigation_tests` must continue to pass, plus new tests for TypeContext-based resolution

  **Must NOT do**:
  - Do NOT remove text fallback functions yet — that's Task 7
  - Do NOT remove any fallback STEPS in the chain — just add TypeContext as a higher-priority path
  - Do NOT change `find_symbol_at_position` (cursor word extraction) — that's still needed
  - Do NOT modify `find_all_symbol_occurrences` (used for references, not definition)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
  - **Reason**: Modifying the definition resolution pipeline and hover system. Needs careful understanding of the fallback chain.

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (after Task 4)
  - **Blocks**: Task 7
  - **Blocked By**: Task 4

  **References**:

  **Pattern References**:
  - `src/lsp/navigation/definition.rs:67-87` — `resolve_symbol_definition()` with 10 fallback steps
  - `src/lsp/navigation/definition.rs:541-549` — `resolve_text_fallback()` (final text-based fallback)
  - `src/lsp/hover/mod.rs:196-362` — `handle_type_context_hover()` already uses TypeContext for type info
  - `src/lsp/helpers.rs` — Location for the conversion helper (has existing `with_document` helper)

  **API/Type References**:
  - `src/type_context.rs` — `DefinitionLocation` struct (from Task 4)
  - `src/lsp/types.rs:13-26` — `Document` struct with `type_context: Option<Arc<TypeContext>>`
  - `lsp_types::Location` — target type for conversion

  **Test References**:
  - `tests/lsp_navigation_tests.rs` — Existing navigation tests (must still pass)

  **Acceptance Criteria**:
  - [ ] Definition resolution checks TypeContext positions before text fallback
  - [ ] `type_context_to_lsp_location` helper exists in `src/lsp/helpers.rs`
  - [ ] `cargo test --test lsp_navigation_tests` → all pass
  - [ ] `cargo test --lib` → all pass
  - [ ] Symbols with TypeContext locations resolve without hitting text fallback

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Navigation tests still pass with new resolution path
    Tool: Bash
    Steps:
      1. cargo test --test lsp_navigation_tests 2>&1 | tail -5
      2. Assert: all pass, 0 failed
    Expected Result: No regression in navigation
    Evidence: Terminal output

  Scenario: Full test suite green
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1 | tail -3
      2. cargo test --test lsp_analysis_tests 2>&1 | tail -3
      3. cargo test --test lsp_completion_tests 2>&1 | tail -3
      4. Assert: all pass
    Expected Result: Zero regressions
    Evidence: Terminal output
  ```

  **Commit**: YES
  - Message: `refactor(lsp): use TypeContext definition locations for symbol resolution`
  - Files: `src/lsp/navigation/definition.rs`, `src/lsp/hover/mod.rs`, `src/lsp/helpers.rs`
  - Pre-commit: `cargo test --lib && cargo test --test lsp_navigation_tests`

---

- [ ] 6. Smarter caching: persist ModuleSystem + skip unchanged files

  **What to do**:
  - **Part A — Persist ModuleSystem across analysis runs**:
    - Currently `analyzer.rs` creates a new `ModuleSystem` for each analysis run (discarding cached modules)
    - Store ModuleSystem in the background analysis worker thread (not in DocumentStore — avoids Send/Sync issues)
    - In `src/lsp/server.rs`, the `background_analysis_worker` function (line 600+) should own a persistent `ModuleSystem`
    - Pass it to `run_compiler_analysis_with_context` (analyzer.rs:67-86) instead of creating new
    - Add content-hash based invalidation: before returning a cached module, check if the source file's content hash has changed
    - ModuleSystem.modules stores `HashMap<String, Program>` — extend to `HashMap<String, (Program, u64)>` where `u64` is the content hash at parse time
    - On cache hit: re-read file, compute hash, compare. If different: re-parse and update cache. If same: return cached.
    - Cap cache size at a reasonable limit (e.g., 200 modules) with LRU eviction or simple eviction of oldest entries

  - **Part B — Skip re-analysis for unchanged files**:
    - In `src/lsp/document_store/document_lifecycle.rs:54-125` (document update handler):
      - The content_hash check already exists (lines 57-76) to skip unchanged content
      - Extend: if content unchanged AND TypeContext exists AND is recent (e.g., < 5s): skip sending AnalysisJob to background thread
    - In `src/lsp/server.rs` background worker:
      - Before running TypeChecker, check if Document.type_context is still valid (content_hash matches)
      - If valid: reuse existing TypeContext, only re-run for changed documents

  **Must NOT do**:
  - Do NOT wrap ModuleSystem in `Arc<Mutex<>>` — TypeChecker has `RefCell<TypeStore>` (not Send/Sync)
  - Do NOT add file-watcher or filesystem monitoring
  - Do NOT build a dependency graph
  - Do NOT implement partial re-analysis (only skip entire unchanged files)
  - Do NOT make ModuleSystem available from the main thread (keep it in background worker only)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
  - **Reason**: Threading and caching changes in the LSP server. Needs careful handling of the background worker thread.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 4)
  - **Blocks**: None
  - **Blocked By**: Tasks 1, 2, 3 (need clean baseline)

  **References**:

  **Pattern References**:
  - `src/lsp/server.rs:600-691` — `background_analysis_worker`: runs in separate thread, receives AnalysisJob, sends AnalysisResult
  - `src/lsp/analyzer.rs:67-86` — `run_compiler_analysis_with_context`: creates ModuleSystem, loads imports, runs TypeChecker
  - `src/lsp/analyzer.rs:131-147` — `load_imports_for_program`: creates new ModuleSystem each time
  - `src/module_system/mod.rs:97-239` — ModuleSystem.load_module with cache check at line 99

  **API/Type References**:
  - `src/module_system/mod.rs:10-15` — ModuleSystem struct: `modules: HashMap<String, Program>`
  - `src/lsp/types.rs:20` — `Document.content_hash: u64` (FNV-1a hash, already computed)
  - `src/lsp/document_store/document_lifecycle.rs:55` — `DEBOUNCE_MS: u128 = 300` (debounce constant)

  **Documentation References**:
  - ModuleSystem is NOT Send/Sync (indirectly, through TypeChecker). Must stay within single thread.
  - Background worker is a dedicated thread — owning ModuleSystem there is safe.

  **Acceptance Criteria**:
  - [ ] ModuleSystem persists across analysis runs in background worker
  - [ ] Cache invalidation via content hash (re-parse if file changed)
  - [ ] Cache size bounded (200 or similar limit)
  - [ ] Unchanged files skip re-analysis when TypeContext is fresh
  - [ ] `cargo test --lib` → all pass
  - [ ] `cargo test --test lsp_analysis_tests` → all pass
  - [ ] `cargo test --test behavioral_tests` → all pass (codegen unaffected)
  - [ ] No deadlocks or race conditions (background worker is single-threaded)

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Module caching works across analysis runs
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1 | tail -3
      2. Assert: all pass, 0 failed
    Expected Result: Caching doesn't break any tests
    Evidence: Terminal output

  Scenario: Full test suite green
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1 | tail -3
      2. cargo test --test lsp_analysis_tests 2>&1 | tail -3
      3. cargo test --test behavioral_tests 2>&1 | tail -3
      4. cargo test --test ptr_ref_tests 2>&1 | tail -3
      5. cargo test --test lsp_completion_tests 2>&1 | tail -3
      6. Assert: all pass
    Expected Result: Zero regressions
    Evidence: Terminal output

  Scenario: No stale cache issues
    Tool: Bash
    Steps:
      1. cargo test --lib -- module 2>&1 | tail -5
      2. Assert: any module-related tests pass
    Expected Result: Module loading still correct with caching
    Evidence: Terminal output
  ```

  **Commit**: YES
  - Message: `perf(lsp): persist ModuleSystem cache and skip unchanged file re-analysis`
  - Files: `src/lsp/server.rs`, `src/lsp/analyzer.rs`, `src/module_system/mod.rs`, `src/lsp/document_store/document_lifecycle.rs`
  - Pre-commit: `cargo test --lib && cargo test --test lsp_analysis_tests`

---

- [ ] 7. Remove redundant text fallback paths

  **What to do**:
  - Now that TypeContext positions are populated (Task 4) and used for resolution (Task 5), remove text-based fallback code that is no longer reachable
  - In `src/lsp/navigation/definition.rs`:
    - Remove or simplify `resolve_text_fallback()` (lines 541-549) — if TypeContext covers all symbols it handled
    - Remove `find_symbol_definition_in_content()` calls that are now superseded by TypeContext lookups
    - Keep any fallback that handles cases TypeContext can't (e.g., symbols in non-typechecked files, stdlib without position data)
  - In `src/lsp/document_store/symbol_extraction.rs`:
    - Remove `extract_symbols_text_fallback()` (lines 24-84) IF AST parsing always succeeds for valid documents
    - If AST parsing can fail for partial documents (incomplete typing), keep the text fallback
  - In `src/lsp/navigation/utils.rs`:
    - Remove `find_symbol_definition_in_content()` (lines 257-303) IF no remaining callers
    - Keep `find_symbol_at_position()` (cursor extraction — still needed)
    - Keep `find_all_symbol_occurrences()` (references feature — still needed)
    - Keep `find_word_in_line()` (still needed for cursor word detection)
  - **CRITICAL**: Remove one fallback at a time. After each removal, run the full test suite. If any test fails, the fallback was still needed — restore it and mark with a TODO explaining why.
  - Use `grep -rn "find_symbol_definition_in_content\|resolve_text_fallback\|extract_symbols_text_fallback"` to find all call sites before removing

  **Must NOT do**:
  - Do NOT remove fallbacks in bulk — one at a time with verification
  - Do NOT remove `find_symbol_at_position` (cursor word extraction — always needed)
  - Do NOT remove `find_all_symbol_occurrences` (references feature — separate from definition)
  - Do NOT remove fallbacks that handle partial/incomplete documents (LSP must work while typing)
  - Do NOT remove stdlib-specific fallbacks that handle symbols not in TypeContext

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: []
  - **Reason**: Careful deletion with verification. Not complex logic, but needs discipline.

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (final task)
  - **Blocks**: None (final)
  - **Blocked By**: Task 5

  **References**:

  **Pattern References**:
  - `src/lsp/navigation/definition.rs:541-549` — `resolve_text_fallback()` — primary removal candidate
  - `src/lsp/navigation/definition.rs:244` — `resolve_member_in_module()` also calls `find_symbol_definition_in_content`
  - `src/lsp/navigation/definition.rs:426` — `resolve_symbol_from_stdlib()` also calls it
  - `src/lsp/navigation/utils.rs:257-303` — `find_symbol_definition_in_content()` — remove if no callers remain
  - `src/lsp/document_store/symbol_extraction.rs:24-84` — `extract_symbols_text_fallback()` — keep if AST fails on partial input

  **Acceptance Criteria**:
  - [ ] `resolve_text_fallback` removed or simplified
  - [ ] No remaining calls to text-based definition search from the happy path
  - [ ] `cargo test --test lsp_navigation_tests` → all pass
  - [ ] `cargo test --lib` → all pass
  - [ ] `cargo test --test lsp_analysis_tests` → all pass
  - [ ] `cargo test --test lsp_completion_tests` → all pass
  - [ ] Fallbacks for partial documents preserved (LSP must work during typing)

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Incremental fallback removal - text_fallback
    Tool: Bash
    Steps:
      1. Remove resolve_text_fallback
      2. cargo test --test lsp_navigation_tests 2>&1 | tail -5
      3. Assert: all pass, or restore if failures
    Expected Result: Navigation works without text fallback
    Evidence: Terminal output

  Scenario: Verify remaining text search functions
    Tool: Bash
    Steps:
      1. grep -rn "find_symbol_definition_in_content" src/lsp/ 2>&1
      2. Assert: 0 remaining calls (or only in kept fallback paths)
    Expected Result: Text search no longer in hot path
    Evidence: grep output

  Scenario: Full test suite green after all removals
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1 | tail -3
      2. cargo test --test lsp_navigation_tests 2>&1 | tail -3
      3. cargo test --test lsp_analysis_tests 2>&1 | tail -3
      4. cargo test --test lsp_completion_tests 2>&1 | tail -3
      5. cargo test --test ptr_ref_tests 2>&1 | tail -3
      6. cargo test --test behavioral_tests 2>&1 | tail -3
      7. Assert: all pass
    Expected Result: Zero regressions
    Evidence: Terminal output
  ```

  **Commit**: YES
  - Message: `refactor(lsp): remove redundant text-based symbol search fallbacks`
  - Files: `src/lsp/navigation/definition.rs`, `src/lsp/navigation/utils.rs`, `src/lsp/document_store/symbol_extraction.rs`
  - Pre-commit: `cargo test --lib && cargo test --test lsp_navigation_tests`

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 0 | No commit (baseline only) | — | — |
| 1 | `fix(parser): emit ! as Operator token so unary not parses correctly` | src/lexer.rs, tests | cargo test --lib |
| 2 | `fix(typechecker): normalize generic type names in UFC method keys` | src/name_utils.rs, src/typechecker/*.rs | cargo test --lib && cargo test --test ptr_ref_tests |
| 3 | `fix(tests): update lsp_completion_tests to use Option<&Document> API` | tests/lsp_completion_tests.rs | cargo test --test lsp_completion_tests |
| 4 | `feat(typechecker): add definition location tracking to TypeContext` | src/type_context.rs, src/typechecker/*.rs | cargo test --lib && cargo test --test behavioral_tests |
| 5 | `refactor(lsp): use TypeContext definition locations for symbol resolution` | src/lsp/navigation/*.rs, src/lsp/helpers.rs | cargo test --lib && cargo test --test lsp_navigation_tests |
| 6 | `perf(lsp): persist ModuleSystem cache and skip unchanged file re-analysis` | src/lsp/server.rs, src/lsp/analyzer.rs, src/module_system/mod.rs | cargo test --lib && cargo test --test lsp_analysis_tests |
| 7 | `refactor(lsp): remove redundant text-based symbol search fallbacks` | src/lsp/navigation/*.rs, src/lsp/document_store/*.rs | cargo test --all |

---

## Success Criteria

### Verification Commands
```bash
# All test suites pass
cargo test --lib                          # Expected: 143+ passed, 0 failed
cargo test --test ptr_ref_tests           # Expected: all passed (including enum methods)
cargo test --test lsp_completion_tests    # Expected: compiles and passes
cargo test --test lsp_navigation_tests    # Expected: all passed
cargo test --test lsp_analysis_tests      # Expected: 41+ passed
cargo test --test behavioral_tests        # Expected: all passed

# ! operator works
# (verified via new parser tests)

# No text fallbacks on hot path
grep -c "resolve_text_fallback" src/lsp/navigation/definition.rs  # Expected: 0 (or minimal)
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass (`cargo test --all`)
- [ ] `!` operator parses universally
- [ ] Enum instance methods resolve on generic types
- [ ] TypeContext carries definition positions
- [ ] ModuleSystem cached across analysis runs with invalidation
- [ ] Text fallbacks removed from definition resolution hot path
