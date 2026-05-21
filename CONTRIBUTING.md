# Contributing to Zen

This repository follows the `rewrite` baseline: one Rust compiler binary lowers
checked Zen programs through the C backend. Broader claims need implementation
and tests in the same change.

## Prerequisites

- Stable Rust
- A C compiler available as `cc` or through the `CC` environment variable

## Local Checks

Run the CI gates locally:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --lib
cargo test --tests
```

## TDD Rule

Failing tests first are required for language work. Before implementing or
documenting parser, semantic, effects, stdlib, codegen, or tooling behavior, add
the smallest failing check that proves the behavior or guards the public claim.

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

Remove a failing test only when the feature leaves the v1 contract and docs
change in the same patch.

## Architecture

The active pipeline is:

```text
source -> lexer -> parser -> module loader -> typechecker -> typed AST -> C backend -> cc
```

The target architecture in [docs/V1_SPEC.md](docs/V1_SPEC.md) adds resolver,
HIR, MIR, effects, monomorphization, ABI, stdlib, and tooling stages as tests
and implementation make them true.

## Documentation

Public docs must be truthful. Untested features are `gated`, `experimental`, or
future work; missing binaries, unsupported targets, and unchecked stdlib APIs are
not complete.
