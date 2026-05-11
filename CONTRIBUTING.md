# Contributing to Zen

This repository currently follows the `rewrite` baseline: one Rust compiler binary
that lowers checked Zen programs to C through the C backend. Keep changes aligned
with that reality unless the change also adds the tests and implementation needed
to make a broader claim true.

## Prerequisites

- Stable Rust
- A C compiler available as `cc` or through the `CC` environment variable

## Local Checks

Run the same checks advertised by CI:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --lib
cargo test --tests
```

## TDD Rule

Failing tests first are required for language work. Before implementing or
documenting a parser, semantic, effects, stdlib, codegen, or tooling change, add
the smallest failing check that proves the intended behavior or guards the public
claim.

Use the narrowest test that covers the risk:

- Parser and lexer changes: unit tests or golden token/AST checks.
- Semantic changes: positive and negative typechecker tests with stable
  diagnostics where possible.
- Effects work: positive and negative checks for legal and illegal `Sync/Async`
  propagation.
- Stdlib work: parse/typecheck/build tests for every adopted stdlib file.
- Codegen work: generated C checks plus executable integration tests.
- Tooling work: assertions that docs, CLI, formatter, or editor claims match
  existing binaries and behavior.

Only remove a failing test when the feature is explicitly removed from the v1
contract and the docs are updated in the same change.

## Architecture

The active pipeline is:

```text
source -> lexer -> parser -> module loader -> typechecker -> typed AST -> C backend -> cc
```

The draft target architecture in [docs/V1_SPEC.md](docs/V1_SPEC.md) adds resolver,
HIR, MIR, effects, monomorphization, ABI, stdlib, and tooling stages over time.
Those are gated until tests and implementation exist.

## Documentation

Public docs must be truthful. If a feature is not covered by tests, describe it as
`gated`, `experimental`, or future work. Do not advertise missing binaries,
unsupported targets, or unchecked stdlib APIs as complete.
