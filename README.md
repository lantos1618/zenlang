# Zen Programming Language

Zen is a work-in-progress systems language compiler. This worktree uses the
`rewrite` branch as the baseline: a small Rust compiler pipeline that parses Zen
source, typechecks it into a typed AST, emits C through the C backend, compiles
the C with `cc`, and runs the result in integration tests.

The current repository is not a complete v1 language. Documentation and examples
should describe only behavior covered by tests, or explicitly mark future work as
gated or experimental.

## Current Baseline

Implemented and tested today:

- Lexer, parser, module loading for local files, typechecker, typed AST, C backend.
- Runtime integration tests for arithmetic, strings, structs, enums, pattern-style
  matches, loops, recursion, mutability, `defer`, casts, and UFCS-style calls.
- CI checks for formatting, clippy, library tests, and integration tests.

Not implemented as stable v1 features yet:

- Real `Sync/Async` effects, typed allocator effects, actor runtime, behavior
  solver, comptime type matching, `build.zen` execution, JSON/YAML IR emission,
  formatter, package manager, alternate backend, or stable ABI/layout contracts.

See [docs/V1_SPEC.md](docs/V1_SPEC.md) for the draft v1 contract, feature matrix,
and required positive/negative test backlog.

## Quick Start

```bash
# Build the compiler
cargo build

# Run the tested integration suite
cargo test --tests

# Run the current CI-equivalent local checks
cargo fmt --check
cargo clippy -- -D warnings
cargo test --lib
cargo test --tests
```

To run a currently tested program manually:

```bash
cargo run -- tests/zen/hello.zen
```

## Repository Layout

```text
src/
  lexer/          tokenization
  parser/         syntax parsing
  module_system/  local file loading and imports
  typechecker/    semantic checks and typed AST construction
  codegen/c/      C backend
tests/
  zen/            executable integration fixtures and expected output
stdlib/           aspirational and experimental Zen stdlib sources
docs/
  V1_SPEC.md      draft v1 contract and feature gates
```

## Development Rule

Language work is TDD-first. Add the failing parser, semantic, effects, stdlib,
codegen, tooling, or documentation assertion before changing implementation or
public claims.

## License

MIT
