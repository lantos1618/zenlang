# Zen Semantic Analysis — current state & target design

Status: design / RFC. Steps 1–2 of the migration are **done** (this branch);
steps 3–4 are proposed and need owner sign-off before implementation.

The semantic layer is the resolver (`src/resolver/`) plus the typechecker
(`src/typechecker/`). It turns the parsed AST into a fully-typed `TypedProgram`
that the C backend lowers. It works end-to-end today — generics, behaviors, and
cross-module imports all function — so this is a *refactor* plan, not a rescue.

---

## 1. Current state (measured)

```
resolver:     1,956 LOC / 17 files
typechecker:  6,279 LOC / 65 files     median 92 lines, max 257, min 4
TOTAL sema:  ~8,200 LOC / 82 files
```

~8,200 LOC for a sema with generics + behaviors + modules is **small-to-normal**.
There is no dead code (clippy `--lib` is clean). The cost is *structure*, not
volume — 82 files is a navigation tax from a past "keep files under a line cap"
split treadmill.

### Confirmed structural slop

| Slop | Evidence | Severity | Status |
|---|---|---|---|
| Misleading `resolver_contract*` name | 915 LOC named "contract" that actually seed imported symbols (a vestige of the deleted "resolver-validation agreement layer"; a prior audit even mistook it for dead code) | High | **Fixed** → `import_seeding` / `info_builders` |
| Monomorphization fragmented | 10 flat `monomorphize_*.rs` modules; near-duplicate `specialize_generic_struct`/`_enum` | Medium | **Fixed** → one `generics/` module, dedup merged |
| **Dual symbol model** | resolver builds `SymbolTable`, then `seed_declaration_info` copies it into the typechecker's own `structs`/`enums`/`functions`/`methods`/`behaviors` maps | High | proposed (step 3) |
| **Stringly-typed dispatch** | `Vec_i64`, `Type.method__Behavior`; method resolution prefix-searches `"Type.method__"` and disambiguates by return type (`call_validation.rs:66`) | High | proposed (step 3) |
| Two substitution mechanisms | a `type_substitutions` stack *field* and a `substitutions: Option<&HashMap>` *param* threading the same concept | Medium | proposed (step 4) |
| Hand-maintained dedup maps | ~5 parallel HashMaps (`specializations_seen`, `*_name_owners`, `specialized_type_args`, …) tracking one concept | Medium | proposed (step 4) |
| `Type::Unknown` threaded everywhere | 34 error-recovery sites continue checking on `Unknown` | Low | proposed |

---

## 2. Target architecture — query-driven, single-store

```
  AST ─►┌──────────────────────────────────────────────────────────────────────┐
        │  ① INTERN  — one pass, builds the ONLY symbol store                     │
        │     Symbols ──► SymbolId (u32)      Types ──► TypeId (u32, interned)    │
        │     names resolved ONCE to ids; NOTHING downstream parses a name again  │
        └───────────────────────────────┬──────────────────────────────────────┘
                                         │   one store, queried — not copied
        ┌────────────────────────────────▼─────────────────────────────────────┐
        │  ② QUERY ENGINE  (memoized, on-demand)                                 │
        │     type_of(SymbolId)                                                  │
        │     resolve_method(recv: TypeId, name) ──► SymbolId   (not a string)   │
        │     impls_behavior(TypeId, BehaviorId) -> bool                         │
        │     specialize(SymbolId, [TypeId]) ──► SymbolId  (memoized = dedup)    │
        │     check_body(SymbolId) -> TypedBody                                  │
        │   each query: pure(inputs) -> output, cached by key. cycle = error.    │
        └───────────────────────────────┬──────────────────────────────────────┘
                                         │
        ┌────────────────────────────────▼─────────────────────────────────────┐
        │  ③ LOWER  — mangling happens HERE and ONLY here, at the codegen edge    │
        │     SymbolId + [TypeId] ──► "Vec_i64"   (a presentation detail)        │
        └───────────────────────────────┬──────────────────────────────────────┘
                                         ▼  TypedProgram
```

### Principles

1. **One store, queried — never copied.** Resolver and typechecker share a
   single symbol store keyed by `SymbolId`. Kill `seed_declaration_info`'s
   copy-into-parallel-maps step; the typechecker queries the store instead.
2. **Resolution returns ids, not names.** `resolve_method` / behavior dispatch
   return a `SymbolId`. No code constructs or parses a mangled string to decide
   *what* to call.
3. **Mangling is a lowering detail.** `SymbolId + [TypeId] → "Vec_i64"` happens
   once, at the codegen boundary. The middle of the compiler never sees it.
4. **The cache is the dedup.** A memoized `specialize(SymbolId, [TypeId])`
   query replaces the ~5 hand-maintained `specializations_seen`-style maps and
   the two substitution mechanisms (substitutions become query inputs, not
   ambient stack state).
5. **One error sentinel.** `Type::Unknown` becomes a single `Error` type that
   poisons quietly and is checked at boundaries, not threaded through 34 sites.
6. **Group by concern, not line count.** Target module shape (~8 dirs):
   `intern/ · types/ · query/ · infer/ · generics/ · traits/ · lower/`.

The single highest-leverage change is **#1 + #2 together**: a shared id-keyed
store with id-returning resolution dissolves both the dual-model slop *and* the
stringly-typed dispatch at once.

---

## 3. Migration path (incremental, each step ships green)

A full query engine is a multi-week, high-risk rewrite. Instead, sequence it so
every step is independently shippable with all tests green:

- [x] **Step 1 — Rename.** `resolver_contract*` → `import_seeding` /
  `info_builders`. Pure rename, zero behavior change.
- [x] **Step 2 — Consolidate generics.** 10 `monomorphize_*.rs` → one
  `generics/` module; merge `specialize_generic_struct`/`_enum`.
- [ ] **Step 3 — Intern + id-based dispatch.** Introduce `SymbolId` in the
  resolver; make `resolve_method`/behavior dispatch return ids; delete the
  typechecker's parallel `structs`/`enums`/`functions`/`methods` maps
  incrementally, querying the store instead. Mangling moves to the lowering
  edge. *(Largest step; do behind a flag, one symbol kind at a time.)*
- [ ] **Step 4 — Memoize specialization.** Replace the parallel dedup HashMaps
  and the two substitution paths with one memoized `specialize` query.

### Explicit non-goals

- No big-bang rewrite. If a step can't land green on its own, it's too big —
  split it.
- Not adopting a full incremental-recompilation framework (e.g. salsa). The
  query *shape* is the win; durable incremental caching is out of scope.
- No behavior/diagnostic changes smuggled into refactor steps. Golden output
  stays byte-identical except where a fix is the explicit purpose of a commit.
