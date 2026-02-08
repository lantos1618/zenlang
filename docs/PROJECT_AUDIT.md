# Zen Compiler Project Audit

Comprehensive audit of the Zen language compiler after the build.zen refactor (commit c25e7859).

---

## Priority 1: Critical Bugs

### P1-1: Comptime `environment.set()` forces `is_mutable: true`
**File**: `src/comptime/environment.rs`
**Issue**: `set()` method always sets `is_mutable: true`, meaning comptime evaluation ignores immutability constraints. Variables declared as immutable can be mutated during comptime.
**Fix**: Preserve the original `is_mutable` flag from the variable definition.

### P1-2: Cast inference silently falls back to I32
**File**: `src/typechecker/inference/calls.rs`
**Issue**: When target type in `cast()` can't be resolved, it silently defaults to `I32` instead of reporting an error. This masks type errors.
**Fix**: Return a proper type error when target type is unresolvable.

### P1-3: `types_match_by_name()` not used in all UFC paths
**File**: `src/typechecker/inference/calls.rs`
**Issue**: The new `types_match_by_name()` fix is only applied in Strategy 1 UFCS. Other type comparison paths (Strategy 2, method resolution) still use structural equality, meaning UFC can still fail in some cases.
**Fix**: Apply name-based matching consistently across all UFC/method resolution paths.

### P1-4: Module loading errors silently swallowed
**File**: `src/module_system/mod.rs`
**Issue**: Multiple places use `let _ = self.load_module(...)` discarding errors. Failed imports produce no diagnostic, making debugging very difficult.
**Fix**: Propagate or log module loading errors.

---

## Priority 2: Build System Issues

### P2-1: Silent error suppression in `parse_build_file()`
**File**: `src/build_system/mod.rs`
**Issue**: Parser/lexer errors during build.zen parsing are caught and return `Ok(BuildConfig)` with empty packages. A malformed build.zen silently produces no packages instead of an error.
**Fix**: Return `Err` with diagnostic info when build.zen parsing fails.

### P2-2: Unused `release` flag in `ExecutableTarget`
**File**: `src/build_system/mod.rs`
**Issue**: `ExecutableTarget` has a `release: bool` field but nothing reads it. The `--release` flag from CLI is not wired through.
**Fix**: Wire `release` flag through to compiler optimization passes, or remove it until needed.

### P2-3: Three duplicate compile functions
**File**: `src/compiler.rs`
**Issue**: `compile_file()`, `compile_string()`, and `run_pipeline()` share 80%+ code with slight variations. Each must be kept in sync manually.
**Fix**: Extract shared compilation logic into a single `compile_inner()` function.

### P2-4: LSP doesn't use project-specific BuildConfig
**File**: `src/lsp/server.rs`
**Issue**: LSP creates ModuleSystem without discovering build.zen. Projects using `std` imports (no `@std`) get false diagnostics in the editor.
**Fix**: Call `BuildConfig::discover()` in LSP workspace initialization and pass PackageMap to ModuleSystem.

### P2-5: 7 duplicate ModuleSystem initialization patterns
**Files**: `src/compiler.rs`, `src/lsp/server.rs`, `src/lsp/analyzer.rs`, tests
**Issue**: ModuleSystem is created from scratch in 7+ places with slightly different configurations. Some include PackageMap, some don't.
**Fix**: Create a `ModuleSystem::new_with_defaults()` factory that handles BuildConfig discovery.

---

## Priority 3: Code Quality

### P3-1: 98 `unwrap()` calls in compiler code
**Files**: Throughout `src/`
**Issue**: Many `unwrap()` calls on Results/Options that could panic on malformed input. Compiler should never panic on user code.
**Key offenders**:
- `src/codegen/llvm/` (41 unwraps)
- `src/typechecker/` (23 unwraps)
- `src/parser/` (18 unwraps)
**Fix**: Replace with proper error propagation (`?` operator) or `.expect("reason")` at minimum.

### P3-2: 7 `panic!()` calls in non-test code
**Files**: `src/codegen/`, `src/typechecker/`
**Issue**: Direct `panic!()` calls crash the compiler instead of producing diagnostics.
**Fix**: Replace with error returns or `unreachable!()` where truly impossible.

### P3-3: 11 swallowed errors with `let _ =`
**Files**: `src/module_system/mod.rs`, `src/compiler.rs`
**Issue**: Error results discarded without logging or handling.
**Fix**: At minimum log the error; ideally propagate it.

### P3-4: Monomorphization uses `format!` for mangled names
**File**: `src/type_system/monomorphization.rs`
**Issue**: Name mangling via string formatting is fragile and could produce collisions with nested generics.
**Fix**: Implement proper name mangling scheme.

---

## Priority 4: Stdlib Issues

### P4-1: `std.zen` exports modules that don't exist
**File**: `stdlib/std.zen`
**Issue**: Exports `math`, `string`, `collections` modules but some functions referenced don't actually exist (e.g., `sqrt`, `abs`, `pow` in math).
**Fix**: Either implement the missing functions or remove the exports.

### P4-2: `testing.zen` references undefined `DynVec` type
**File**: `stdlib/testing.zen`
**Issue**: Uses `DynVec` type that isn't defined or imported anywhere in stdlib.
**Fix**: Replace with `Vec` or define `DynVec`.

### P4-3: 29 missing intrinsic wrappers in `compiler.zen`
**File**: `stdlib/compiler.zen`
**Issue**: `compiler.zen` is the sole authorized wrapper for `@builtin.*` calls, but only wraps ~15 intrinsics. Many intrinsics used in stdlib (syscalls, memory ops) have no wrapper.
**Fix**: Add wrappers for all intrinsics that stdlib files need.

### P4-4: `@builtin` leaks in stdlib files
**Files**: Several stdlib files outside `compiler.zen`
**Issue**: Some stdlib files still call `@builtin.*` directly instead of going through `compiler.zen` wrappers (partially fixed in Phase 0A but may have been missed).
**Fix**: Audit all stdlib files and route through `compiler.zen`.

---

## Priority 5: Parser/Typechecker Polish

### P5-1: Loop parser ambiguity
**File**: `src/parser/statements.rs`
**Issue**: `loop condition {` is ambiguous when condition ends with an identifier, because `identifier {` is parsed as struct literal. Currently worked around in examples by using closure form.
**Fix**: Add lookahead or different syntax to distinguish loop conditions from struct literals.

### P5-2: Pattern match arm type inference
**File**: `src/typechecker/`
**Issue**: Pattern match arms without explicit `return` don't properly propagate their expression types for enum payloads. Requires explicit `return` in each arm.
**Fix**: Implement proper tail-expression inference in match arms.

### P5-3: Inline member access chains not supported in imports
**File**: `src/parser/program.rs`
**Issue**: `std.math.sqrt()` works in import destructuring but not as inline expression. Must import module first, then call function.
**Fix**: Support dotted path resolution for function calls, not just imports.

---

## Priority 6: Test Coverage Gaps

### P6-1: No integration tests for build.zen workflow
**Issue**: The new build.zen discovery and PackageMap integration has unit tests but no end-to-end integration test.
**Fix**: Add integration test that creates a temp project with build.zen and verifies compilation.

### P6-2: No tests for error paths in build_system
**Issue**: Error handling in build.zen parsing is untested.
**Fix**: Add tests for malformed build.zen, missing build.zen, circular imports.

### P6-3: UFC tests only cover basic cases
**Issue**: The `types_match_by_name` fix is tested implicitly but not explicitly.
**Fix**: Add unit tests for UFC with generics, enums, nested structs.

### P6-4: Comptime interpreter lacks coverage
**Issue**: Comptime evaluation of package-prefixed paths (from build.zen) is untested.
**Fix**: Add comptime tests with PackageMap-resolved modules.

---

## Recommended Fix Order

**Batch 1 (Critical, ~2h)**: P1-1, P1-2, P1-3, P1-4
**Batch 2 (Build System, ~2h)**: P2-1, P2-3, P2-4, P2-5
**Batch 3 (Stdlib, ~1.5h)**: P4-1, P4-2, P4-3, P4-4
**Batch 4 (Tests, ~1.5h)**: P6-1, P6-2, P6-3, P6-4
**Batch 5 (Polish, ~2h)**: P3-1 (top offenders), P3-2, P5-1, P5-2

Total: ~20 discrete issues, ~9 hours estimated work across parallel agents.
