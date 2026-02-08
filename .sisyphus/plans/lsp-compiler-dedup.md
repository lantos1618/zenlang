# LSP-Compiler Deduplication: Full Sweep

## TL;DR

> **Quick Summary**: Eliminate duplication between the LSP and compiler by unifying variable type inference, consolidating type name extraction, and moving pattern exhaustiveness + allocator validation from LSP into the compiler's typechecker. Keep LSP fallbacks alive for async timing window.
>
> **Deliverables**:
> - Characterization tests for pattern checking + allocator validation (currently zero coverage)
> - `AstType::base_name()` method eliminating 4 duplicate type-name extraction sites
> - Unified variable type inference (2 impls → 1)
> - Pattern exhaustiveness checking moved from LSP to compiler typechecker
> - Allocator validation moved from LSP to compiler typechecker
> - LSP adapter layer calling compiler validations instead of own implementations
>
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: Task 0 → Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Task 6

---

## Context

### Original Request
User asked "how much does the LSP rewrite or duplicate logic that is in the compiler, we should be leveraging that." Chose "all of the above" for full sweep.

### Interview Summary
**Key Discussions**:
- Phase 2 codebase cleanup is complete (AstFields, handler dedup, codegen dedup)
- Two explore agents found 6 duplication areas
- Two deep-dive agents analyzed TypeContext architecture and inference fallbacks

**Research Findings**:
- TypeContext stores types ONLY, no position/location data — text fallbacks serve a real purpose
- TypeContext is populated asynchronously — timing window where it's unavailable
- LSP fallbacks exist intentionally for this timing window (not legacy code)
- LSP is already 80% AST-based for symbol resolution; text search is only ~20% fallback
- Pattern exhaustiveness + allocator validation have ZERO test coverage
- Pattern checking searches across ALL documents (multi-file) — not trivial to move

### Metis Review
**Identified Gaps** (addressed):
- Variable type inference has 3 paths, not 2 (added TypeQuery.find_variable_type)
- Type name extraction has 4 sites, not 3 (added navigation/type_definition.rs extract_type_name)
- Pattern checking has multi-document dependency (find_missing_variants searches all docs)
- Zero test coverage for code being moved — must write tests BEFORE moving
- `AstType::base_name()` doesn't fully replace `get_type_name()` — they do different things
- Diagnostic severity must not change during move

---

## Work Objectives

### Core Objective
Consolidate duplicated logic between LSP and compiler so both share single implementations. Move validation logic (pattern exhaustiveness, allocator) into the compiler where both CLI builds and LSP benefit.

### Concrete Deliverables
- `src/ast/types.rs` — add `base_name()` method
- `src/lsp/hover/inference.rs` — refactor to use unified inference
- `src/lsp/document_store/variable_extraction.rs` — delegate to unified inference
- `src/typechecker/validation.rs` — add pattern exhaustiveness + allocator checks
- `src/lsp/analyzer.rs` — call compiler validations instead of own implementations
- `src/lsp/pattern_checking.rs` — thin adapter calling compiler's implementation

### Definition of Done
- [ ] `cargo check -p zen` passes
- [ ] `cargo test --lib` passes (143+ tests, 0 failures)
- [ ] All new characterization tests pass
- [ ] Pattern exhaustiveness fires from compiler (not just LSP)
- [ ] Allocator validation fires from compiler (not just LSP)
- [ ] LSP diagnostic output identical before/after

### Must Have
- Characterization tests written BEFORE any code moves
- LSP fallback chain kept intact (marked with TODO but not deleted)
- Diagnostic severity preserved (WARNING stays WARNING, ERROR stays ERROR)
- Pattern exhaustiveness works with cross-module enums after move
- Every task independently testable and committable

### Must NOT Have (Guardrails)
- Do NOT touch async architecture (background thread, server.rs main loop)
- Do NOT remove LSP fallback paths — mark deprecated with TODO
- Do NOT remove `TypeQuery::infer_literal_type()` — it's a 24-line intentional fallback
- Do NOT change diagnostic severity levels during moves
- Do NOT touch DocumentStore internals or module system
- Do NOT add position data to TypeContext (that's a separate architectural change)
- Do NOT combine multiple dedup areas into single tasks
- Do NOT refactor `format_type()` — it's already centralized
- Do NOT make typechecker produce LSP diagnostics directly — keep adapter layer

---

## Verification Strategy

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: Tests-before (characterization tests) then tests-after
- **Framework**: cargo test

### Agent-Executed QA Scenarios (MANDATORY)

```
Scenario: All existing tests pass after each task
  Tool: Bash
  Steps:
    1. cargo test --lib 2>&1
    2. Assert: 143+ passed, 0 failed
  Expected Result: Zero regressions
  Evidence: Terminal output

Scenario: Compiler catches non-exhaustive patterns (after Task 5)
  Tool: Bash
  Preconditions: Compiler built with validation changes
  Steps:
    1. Create test file with non-exhaustive match on Option<i32>
    2. Run typechecker on file
    3. Assert: warning about missing None arm
  Expected Result: Compiler produces pattern exhaustiveness warning
  Evidence: Compiler output

Scenario: Compiler catches missing allocators (after Task 4)
  Tool: Bash
  Preconditions: Compiler built with validation changes
  Steps:
    1. Create test file using Vec<i32>.new() without allocator
    2. Run typechecker on file
    3. Assert: error about missing allocator
  Expected Result: Compiler produces allocator validation error
  Evidence: Compiler output

Scenario: LSP diagnostics identical before/after
  Tool: Bash
  Steps:
    1. cargo test --test lsp_navigation_tests 2>&1
    2. Assert: all pass, 0 failures
  Expected Result: LSP behavior preserved
  Evidence: Terminal output
```

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — independent tasks):
├── Task 0: Write characterization tests for pattern checking + allocator validation
└── Task 1: Add AstType::base_name() method

Wave 2 (After Wave 1):
├── Task 2: Unify variable type inference
└── Task 3: Consolidate type name extraction sites

Wave 3 (After Wave 2 — sequential, riskier):
├── Task 4: Move allocator validation to compiler (self-contained)
└── Task 5: Move pattern exhaustiveness to compiler (multi-document challenge)

Wave 4 (After Wave 3):
└── Task 6: Clean up LSP adapter layer + remove dead code
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 0 | None | 4, 5 | 1 |
| 1 | None | 3 | 0 |
| 2 | None | 6 | 0, 1 |
| 3 | 1 | 6 | 2 |
| 4 | 0 | 6 | 5 |
| 5 | 0 | 6 | 4 |
| 6 | 2, 3, 4, 5 | None | None (final) |

---

## TODOs

- [ ] 0. Write characterization tests for pattern checking and allocator validation

  **What to do**:
  - Read `src/lsp/pattern_checking.rs` to understand what cases it checks
  - Read `src/lsp/analyzer.rs` `check_allocator_usage()` to understand what it validates
  - Create test file `tests/validation_tests.rs` (or add to existing test module)
  - Write tests that capture CURRENT behavior:
    - Pattern exhaustiveness: non-exhaustive enum match → warning produced
    - Pattern exhaustiveness: exhaustive enum match → no warning
    - Pattern exhaustiveness: Option<T> match missing None → warning
    - Pattern exhaustiveness: match with wildcard → no warning
    - Allocator validation: Vec<T>.new() without allocator → error produced
    - Allocator validation: Vec<T>.new(allocator) → no error
    - Allocator validation: String without allocator → error produced
  - These tests lock in behavior so we can safely move the code later

  **Must NOT do**:
  - Do NOT change any validation logic — only add tests
  - Do NOT modify pattern_checking.rs or analyzer.rs
  - Tests should exercise the LSP analyzer pipeline to capture current behavior

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
    
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: Tasks 4, 5
  - **Blocked By**: None

  **References**:
  - `src/lsp/pattern_checking.rs:10-77` — pattern exhaustiveness checking entry point
  - `src/lsp/pattern_checking.rs:80-170` — `analyze_match_expression` and variant tracking
  - `src/lsp/pattern_checking.rs:212-301` — `find_missing_variants` (searches docs + workspace + stdlib)
  - `src/lsp/analyzer.rs:150-270` — `check_allocator_usage()` with `stdlib_types().requires_allocator()`
  - `src/lsp/analyzer.rs:273-296` — callback setup connecting pattern checking to document analysis
  - `tests/` — existing test files for pattern reference

  **Acceptance Criteria**:
  - [ ] Test file created with 7+ characterization tests
  - [ ] Tests capture both positive (violation detected) and negative (no violation) cases
  - [ ] `cargo test --lib` passes (143+ tests including new ones)
  - [ ] Tests fail if pattern_checking.rs or allocator validation is removed (proves they exercise the code)

  **Agent-Executed QA Scenarios**:
  ```
  Scenario: Characterization tests compile and pass
    Tool: Bash
    Steps:
      1. cargo test --lib -- validation 2>&1
      2. Assert: all new tests pass
    Expected Result: Tests lock in current behavior
    Evidence: Terminal output
  ```

  **Commit**: YES
  - Message: `test(validation): add characterization tests for pattern exhaustiveness and allocator validation`
  - Files: `tests/validation_tests.rs` or relevant test module
  - Pre-commit: `cargo test --lib`

- [ ] 1. Add `AstType::base_name()` method

  **What to do**:
  - In `src/ast/types.rs`, add a method to the `impl AstType` block:
    ```rust
    pub fn base_name(&self) -> Option<&str> {
        match self {
            AstType::Struct { name, .. } => Some(name.as_str()),
            AstType::Generic { name, .. } => Some(name.as_str()),
            AstType::Enum { name, .. } => Some(name.as_str()),
            AstType::EnumType(name) => Some(name.as_str()),
            _ => self.primitive_name(),
        }
    }
    ```
  - Check if `primitive_name()` already exists on AstType — if yes, `base_name()` extends it. If no, implement both.
  - NOTE: This does NOT replace `get_type_name()` in semantic_completion.rs — that function does more (handles Display fallback). `base_name()` is a simpler primitive for type lookups.

  **Must NOT do**:
  - Do NOT replace `get_type_name()` with `base_name()` — they serve different purposes
  - Do NOT modify existing AstType methods

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 0)
  - **Blocks**: Task 3
  - **Blocked By**: None

  **References**:
  - `src/ast/types.rs` — AstType enum and existing impl block
  - `src/lsp/semantic_completion.rs:58-68` — `get_type_name()` for comparison (NOT to be replaced)
  - `src/lsp/navigation/type_definition.rs:8-21` — `extract_type_name()` (string parsing approach)

  **Acceptance Criteria**:
  - [ ] `AstType::base_name()` method added
  - [ ] Returns `Some("name")` for Struct, Generic, Enum, EnumType
  - [ ] Returns primitive name for I32, Bool, etc.
  - [ ] Returns `None` for types without a name
  - [ ] `cargo check -p zen` passes

  **Agent-Executed QA Scenarios**:
  ```
  Scenario: New method compiles
    Tool: Bash
    Steps:
      1. cargo check -p zen 2>&1
      2. Assert: exit code 0
    Expected Result: Clean compilation
    Evidence: Terminal output
  ```

  **Commit**: YES
  - Message: `feat(ast): add AstType::base_name() for centralized type name extraction`
  - Files: `src/ast/types.rs`

- [ ] 2. Unify variable type inference

  **What to do**:
  - Read the 3 variable type inference paths:
    1. `src/lsp/hover/inference.rs:69-116` — `infer_variable_type()` (comprehensive)
    2. `src/lsp/document_store/variable_extraction.rs:80-95` — `infer_variable_type()` (minimal)
    3. `src/lsp/type_query.rs:49-57` — `find_variable_type()` (canonical TypeContext path)
  - Create a single unified function in `type_query.rs`:
    ```rust
    pub fn infer_variable_type_unified(&self, name: &str, ast: &[Declaration]) -> Option<AstType> {
        // 1. Check TypeContext first (canonical path)
        if let Some(ty) = self.find_variable_type(name) { return Some(ty); }
        // 2. Walk AST for declaration with explicit type
        // 3. Fallback to literal inference from initializer
    }
    ```
  - Update `hover/inference.rs` to call unified function + format output
  - Update `variable_extraction.rs` to call unified function
  - Use `lsp_find_references` before modifying to map all callers

  **Must NOT do**:
  - Do NOT remove the fallback chain in hover/mod.rs
  - Do NOT remove `infer_literal_type()` — it's the lightweight fallback
  - Do NOT change hover output format — just consolidate the lookup logic

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 3)
  - **Blocks**: Task 6
  - **Blocked By**: None

  **References**:
  - `src/lsp/hover/inference.rs:69-116` — comprehensive inference (walks AST + TypeQuery + symbols)
  - `src/lsp/document_store/variable_extraction.rs:80-95` — minimal inference (explicit type or literal)
  - `src/lsp/type_query.rs:49-57` — canonical TypeContext path (find_variable_type)
  - `src/lsp/type_query.rs:130-160` — `infer_literal_type()` lightweight fallback
  - `src/lsp/hover/mod.rs:70-184` — hover fallback chain (DO NOT remove)

  **Acceptance Criteria**:
  - [ ] Single `infer_variable_type_unified` function in type_query.rs
  - [ ] `hover/inference.rs` delegates to unified function
  - [ ] `variable_extraction.rs` delegates to unified function
  - [ ] `cargo check -p zen` passes
  - [ ] `cargo test --lib` passes (143+ tests)

  **Agent-Executed QA Scenarios**:
  ```
  Scenario: Hover still works on variables
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1
      2. Assert: all pass including any hover-related tests
    Expected Result: Hover behavior preserved
    Evidence: Terminal output
  ```

  **Commit**: YES
  - Message: `refactor(lsp): unify variable type inference into single TypeQuery method`
  - Files: `src/lsp/type_query.rs`, `src/lsp/hover/inference.rs`, `src/lsp/document_store/variable_extraction.rs`
  - Pre-commit: `cargo test --lib`

- [ ] 3. Consolidate type name extraction sites

  **What to do**:
  - Now that `AstType::base_name()` exists (Task 1), update call sites that duplicate this logic:
    1. `src/lsp/semantic_completion.rs:58-68` — `get_type_name()`: Replace the `match` body with `base_name()` call + Display fallback. Keep the function as a wrapper since it does more than base_name.
    2. `src/lsp/navigation/type_definition.rs:8-21` — `extract_type_name()`: If it can use `SymbolInfo.type_info` + `base_name()` instead of parsing detail strings, refactor. If not, leave a TODO.
    3. Any inline type-name extraction in `hover/mod.rs` or `type_query.rs` — use `base_name()` where applicable
  - Use `ast_grep_search` to find all sites that match type names from AstType
  - For each site, evaluate whether `base_name()` fits or if the existing code does something different

  **Must NOT do**:
  - Do NOT force `base_name()` where it doesn't fit
  - Do NOT delete `get_type_name()` — simplify its internals instead
  - Do NOT change any public API signatures

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 2)
  - **Blocks**: Task 6
  - **Blocked By**: Task 1

  **References**:
  - `src/ast/types.rs` — new `base_name()` method (Task 1)
  - `src/lsp/semantic_completion.rs:58-68` — `get_type_name()` to simplify
  - `src/lsp/navigation/type_definition.rs:8-21` — `extract_type_name()` parsing strings
  - `src/lsp/type_query.rs:189-208` — `resolve_receiver_in_function()` inline extraction

  **Acceptance Criteria**:
  - [ ] `get_type_name()` uses `base_name()` internally
  - [ ] At least 2 other call sites simplified
  - [ ] `cargo check -p zen` passes
  - [ ] `cargo test --lib` passes

  **Commit**: YES
  - Message: `refactor(lsp): use AstType::base_name() to consolidate type name extraction`
  - Files: `src/lsp/semantic_completion.rs`, `src/lsp/navigation/type_definition.rs`, possibly others
  - Pre-commit: `cargo test --lib`

- [ ] 4. Move allocator validation from LSP to compiler

  **What to do**:
  - Read `src/lsp/analyzer.rs:150-270` — `check_allocator_usage()` thoroughly
  - Read `src/typechecker/validation.rs` — understand existing validation pattern
  - Create new function in `src/typechecker/validation.rs`:
    ```rust
    pub fn check_allocator_usage(program: &[Declaration]) -> Vec<AllocatorWarning> {
        // Move logic from analyzer.rs, adapted to return compiler-style results
        // Use stdlib_types().requires_allocator() (check where this is defined)
    }
    ```
  - Keep diagnostic severity as ERROR (matching current behavior)
  - Update `src/lsp/analyzer.rs` to call compiler's version instead of own
  - Remove the LSP-specific implementation (now just an adapter that converts results to LSP Diagnostic)
  - IMPORTANT: Check where `stdlib_types()` is defined and ensure it's accessible from typechecker

  **Must NOT do**:
  - Do NOT change diagnostic severity (ERROR stays ERROR)
  - Do NOT change what counts as a violation
  - Do NOT make the typechecker produce LSP Diagnostic types directly — return compiler result types

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Task 5)
  - **Blocks**: Task 6
  - **Blocked By**: Task 0

  **References**:
  - `src/lsp/analyzer.rs:150-270` — current allocator validation to move
  - `src/typechecker/validation.rs` — target location (existing validation pattern)
  - `src/lsp/analyzer.rs:68-87` — pattern for calling compiler analysis from LSP
  - Grep for `stdlib_types` and `requires_allocator` to find the registry

  **Acceptance Criteria**:
  - [ ] `src/typechecker/validation.rs` has `check_allocator_usage()`
  - [ ] `src/lsp/analyzer.rs` calls compiler's version
  - [ ] LSP-specific allocator validation removed (only adapter remains)
  - [ ] Characterization tests from Task 0 still pass
  - [ ] `cargo check -p zen` passes
  - [ ] `cargo test --lib` passes

  **Agent-Executed QA Scenarios**:
  ```
  Scenario: Allocator validation fires from compiler
    Tool: Bash
    Steps:
      1. cargo test --lib -- validation 2>&1
      2. Assert: allocator characterization tests pass
    Expected Result: Validation works from compiler side
    Evidence: Terminal output
  ```

  **Commit**: YES
  - Message: `refactor(typechecker): move allocator validation from LSP to compiler`
  - Files: `src/typechecker/validation.rs`, `src/lsp/analyzer.rs`
  - Pre-commit: `cargo test --lib`

- [ ] 5. Move pattern exhaustiveness checking from LSP to compiler

  **What to do**:
  - This is the RISKIEST task. Read thoroughly before starting:
    - `src/lsp/pattern_checking.rs` (all 302 lines)
    - `src/lsp/analyzer.rs:273-296` — callback setup
  - **KEY CHALLENGE**: `find_missing_variants()` searches across ALL documents + workspace + stdlib for enum definitions. The compiler's typechecker operates on a SINGLE compilation unit. You need to ensure the typechecker has access to enum definitions from other modules.
  - Investigate: Does `TypeContext` or the typechecker's scope already have cross-module enum info? Check how `import` brings enum definitions into scope.
  - Create functions in `src/typechecker/validation.rs`:
    ```rust
    pub fn check_pattern_exhaustiveness(
        program: &[Declaration],
        enum_registry: &HashMap<String, Vec<String>>,  // enum_name -> variant_names
    ) -> Vec<PatternWarning>
    ```
  - The `enum_registry` parameter lets the caller (LSP or compiler) provide enum definitions from whatever source they have
  - Keep diagnostic severity as WARNING
  - Update `src/lsp/pattern_checking.rs` to be a thin adapter that:
    1. Builds enum_registry from documents + workspace + stdlib symbols
    2. Calls compiler's `check_pattern_exhaustiveness()`
    3. Converts results to LSP Diagnostics

  **Must NOT do**:
  - Do NOT change diagnostic severity (WARNING stays WARNING)
  - Do NOT remove the multi-document enum search — just move it to the adapter
  - Do NOT try to make the compiler search across documents — that's LSP's job
  - Do NOT change which patterns are considered exhaustive

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Task 4)
  - **Blocks**: Task 6
  - **Blocked By**: Task 0

  **References**:
  - `src/lsp/pattern_checking.rs:10-77` — `check_pattern_exhaustiveness()` entry
  - `src/lsp/pattern_checking.rs:80-170` — `analyze_match_expression()` core logic
  - `src/lsp/pattern_checking.rs:212-301` — `find_missing_variants()` multi-doc enum search
  - `src/lsp/analyzer.rs:273-296` — callback setup
  - `src/typechecker/validation.rs` — target location

  **Acceptance Criteria**:
  - [ ] `src/typechecker/validation.rs` has `check_pattern_exhaustiveness()`
  - [ ] Pure function taking AST + enum registry (no LSP dependencies)
  - [ ] `src/lsp/pattern_checking.rs` is thin adapter (builds registry + calls compiler + converts)
  - [ ] Characterization tests from Task 0 still pass
  - [ ] `cargo check -p zen` passes
  - [ ] `cargo test --lib` passes

  **Agent-Executed QA Scenarios**:
  ```
  Scenario: Pattern exhaustiveness fires from compiler
    Tool: Bash
    Steps:
      1. cargo test --lib -- validation 2>&1
      2. Assert: pattern characterization tests pass
    Expected Result: Validation works from compiler side
    Evidence: Terminal output

  Scenario: Multi-file enum checking still works
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1
      2. Assert: all tests pass including any pattern-related
    Expected Result: Cross-module patterns still checked
    Evidence: Terminal output
  ```

  **Commit**: YES
  - Message: `refactor(typechecker): move pattern exhaustiveness checking from LSP to compiler`
  - Files: `src/typechecker/validation.rs`, `src/lsp/pattern_checking.rs`, `src/lsp/analyzer.rs`
  - Pre-commit: `cargo test --lib`

- [ ] 6. Clean up LSP adapter layer + remove dead code

  **What to do**:
  - After Tasks 2-5, audit all changed files for dead code
  - Remove any now-unused helper functions from:
    - `src/lsp/hover/inference.rs` — remove functions replaced by unified inference
    - `src/lsp/document_store/variable_extraction.rs` — remove old `infer_variable_type` if fully replaced
    - `src/lsp/analyzer.rs` — remove old allocator/pattern code replaced by compiler calls
  - Add TODO comments on remaining fallback paths:
    ```rust
    // TODO: Remove fallback when TypeContext is guaranteed available (requires sync analysis)
    ```
  - Verify no unused imports remain
  - Run `cargo clippy -p zen` to catch any issues

  **Must NOT do**:
  - Do NOT remove the hover fallback chain
  - Do NOT remove `infer_literal_type()`
  - Do NOT remove text-based symbol search (still needed for parse failures)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 4 (final)
  - **Blocks**: None
  - **Blocked By**: Tasks 2, 3, 4, 5

  **References**:
  - All files modified in Tasks 2-5
  - `src/lsp/hover/mod.rs:70-184` — fallback chain (DO NOT remove)

  **Acceptance Criteria**:
  - [ ] No dead code in modified files
  - [ ] TODO comments on remaining fallback paths
  - [ ] `cargo check -p zen` passes with no warnings
  - [ ] `cargo test --lib` passes (143+ tests)
  - [ ] `cargo clippy -p zen` clean (or only pre-existing warnings)

  **Agent-Executed QA Scenarios**:
  ```
  Scenario: Clean build with no warnings
    Tool: Bash
    Steps:
      1. cargo check -p zen 2>&1
      2. Assert: no warnings from our modified files
    Expected Result: Clean codebase
    Evidence: Terminal output

  Scenario: Full test suite
    Tool: Bash
    Steps:
      1. cargo test --lib 2>&1
      2. Assert: 143+ passed, 0 failed
    Expected Result: Zero regressions
    Evidence: Terminal output
  ```

  **Commit**: YES
  - Message: `refactor(lsp): clean up dead code and add TODO markers for remaining fallbacks`
  - Files: Various LSP files
  - Pre-commit: `cargo test --lib`

---

## Commit Strategy

| After Task | Message | Verification |
|------------|---------|--------------|
| 0 | `test(validation): add characterization tests for pattern exhaustiveness and allocator validation` | cargo test --lib |
| 1 | `feat(ast): add AstType::base_name() for centralized type name extraction` | cargo check |
| 2 | `refactor(lsp): unify variable type inference into single TypeQuery method` | cargo test --lib |
| 3 | `refactor(lsp): use AstType::base_name() to consolidate type name extraction` | cargo test --lib |
| 4 | `refactor(typechecker): move allocator validation from LSP to compiler` | cargo test --lib |
| 5 | `refactor(typechecker): move pattern exhaustiveness checking from LSP to compiler` | cargo test --lib |
| 6 | `refactor(lsp): clean up dead code and add TODO markers for remaining fallbacks` | cargo test --lib |

---

## Success Criteria

### Verification Commands
```bash
cargo check -p zen              # Expected: clean compile
cargo test --lib                 # Expected: 143+ passed, 0 failed
cargo test --lib -- validation   # Expected: all characterization tests pass
```

### Final Checklist
- [ ] Variable type inference: 2 impls → 1 unified function
- [ ] Type name extraction: uses `base_name()` where applicable
- [ ] Allocator validation: lives in compiler, LSP calls it
- [ ] Pattern exhaustiveness: lives in compiler, LSP adapts it
- [ ] Characterization tests exist and pass
- [ ] LSP fallback chain intact (marked TODO but functional)
- [ ] No dead code in modified files
- [ ] All 143+ tests pass
