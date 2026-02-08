# Zen Compiler Project Audit

Comprehensive audit of the Zen language compiler after the build.zen refactor.
Last updated: 2026-02-08 (after commit 5670c6e4)

---

## Completed Fixes

| ID | Issue | Commit |
|----|-------|--------|
| P1-1 | Comptime `environment.set()` forced `is_mutable: true` | 5670c6e4 |
| P1-2 | Cast with bad second argument had no error message | 5670c6e4 |
| P1-3 | `types_match_by_name()` audited — already in correct place, made `pub(crate)` | 5670c6e4 |
| P1-4 | 6 swallowed module load errors → eprintln logging | 5670c6e4 |
| P2-1 | `parse_build_file()` silently returned Ok on errors → now returns Err | 5670c6e4 |
| P2-3 | Duplicate PackageMap setup → extracted `create_module_system()` | 5670c6e4 |
| P2-4 | LSP didn't discover build.zen → now uses `with_build_config()` | 5670c6e4 |
| P2-5 | 7 duplicate ModuleSystem inits → `ModuleSystem::with_build_config()` factory | 5670c6e4 |
| P4-2 | `testing.zen` used undefined DynVec → replaced with Vec | 5670c6e4 |
| P4-3 | @builtin wrapper audit → confirmed already clean | 5670c6e4 |
| P4-4 | @builtin leak audit → confirmed no leaks | 5670c6e4 |
| P4-1 | std.zen exports audit → confirmed all modules exist | 5670c6e4 |

---

## Remaining Issues

### Priority A: Language Ergonomics (affects user experience)

#### A1: Pattern match arms require explicit `return`
**File**: `src/typechecker/`
**Impact**: High — every match arm needs `return`, which is ugly and surprising.
```zen
// Currently required:
shape ? | .Circle(r) { return 3.14 * r * r }
// Should work:
shape ? | .Circle(r) { 3.14 * r * r }
```
**Fix**: Implement tail-expression inference in match arms. The last expression in a block should be its implicit return value.

#### A2: Loop parser ambiguity with `loop condition {`
**File**: `src/parser/statements.rs`
**Impact**: Medium — `loop i < n {` fails when condition ends with identifier because `identifier {` parses as struct literal.
**Workaround**: Use `loop(() { ... })` closure form or `loop { condition ? | true { break }; ... }`.
**Fix**: Add parser lookahead to distinguish `loop expr { body }` from struct literals.

#### A3: Inline dotted paths don't work as expressions
**File**: `src/parser/program.rs`
**Impact**: Medium — can't write `std.math.sqrt(x)` inline, must import first.
**Fix**: Support dotted path resolution for function calls, not just import destructuring.

### Priority B: Compiler Robustness

#### B1: 98 `unwrap()` calls in non-test code
**Files**: `src/codegen/llvm/` (41), `src/typechecker/` (23), `src/parser/` (18)
**Impact**: Compiler panics on malformed input instead of producing diagnostics.
**Fix**: Replace with `?` or `.expect("reason")`. Start with codegen (most frequent).

#### B2: 7 `panic!()` calls in non-test code
**Files**: `src/codegen/`, `src/typechecker/`
**Impact**: Same as B1.
**Fix**: Replace with error returns or `unreachable!()`.

#### B3: Remaining swallowed errors
**Files**: `src/module_system/mod.rs`, `src/compiler.rs`
**Impact**: ~5 remaining `let _ =` patterns beyond the 6 already fixed.
**Fix**: Log or propagate.

#### B4: Monomorphization name mangling uses `format!`
**File**: `src/type_system/monomorphization.rs`
**Impact**: Could produce collisions with deeply nested generics.
**Fix**: Proper mangling scheme (e.g., Itanium-style).

### Priority C: Build System Polish

#### C1: Unused `release` flag in `ExecutableTarget`
**File**: `src/build_system/mod.rs`
**Impact**: Low — `--release` flag accepted but not wired to optimization.
**Fix**: Wire to LLVM optimization level or remove field.

### Priority D: Test Coverage

#### D1: No integration test for build.zen workflow
**Fix**: Temp project with build.zen → compile → verify output.

#### D2: No tests for build.zen error paths
**Fix**: Malformed build.zen, missing build.zen, bad imports.

#### D3: UFC edge case tests
**Fix**: UFC with generics, enums, nested structs, multi-level.

#### D4: Comptime + PackageMap tests
**Fix**: Comptime evaluation with package-prefixed module paths.

---

## Recommended Next Steps

1. **A1 (tail-expression inference)** — biggest language ergonomics win
2. **B1/B2 (unwrap/panic reduction)** — most important for compiler stability
3. **D1-D4 (test coverage)** — protect against regressions
4. **A2 (loop ambiguity)** — requires design decision on syntax
