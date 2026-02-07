# Contributing to Zen

Thank you for your interest in contributing to the Zen programming language!

## Getting Started

### Prerequisites

- Rust (stable, 1.75+)
- LLVM 18 (for code generation)
- Git

### Building

```bash
# Clone the repository
git clone https://github.com/anthropics/zenlang.git
cd zenlang

# Build the compiler
cargo build --release

# Run tests
cargo test --all

# Run clippy
cargo clippy
```

### Project Structure

```
src/
├── ast/              # Abstract Syntax Tree definitions
├── parser/           # Lexer and parser (5,800+ LOC)
├── typechecker/      # Type checking and inference (4,200+ LOC)
├── codegen/llvm/     # LLVM code generation (11,700+ LOC)
├── lsp/              # Language Server Protocol (12,000+ LOC)
├── type_system/      # Generic type resolution
├── module_system/    # Cross-module imports
├── comptime/         # Compile-time evaluation
├── error.rs          # Error types and handling
└── compiler.rs       # Main compilation pipeline

stdlib/               # Standard library (Zen source)
tests/                # Integration tests
examples/             # Example programs
docs/                 # Documentation
```

## Development Workflow

### 1. Find Something to Work On

- Check `docs/reviews/` for current priorities
- Look at `docs/ROADMAP.md` for planned features
- Search for `TODO` comments in the codebase
- Check open issues

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 3. Make Changes

- Follow existing code style
- Add tests for new functionality
- Update documentation if needed
- Keep commits atomic and well-described

### 4. Test Your Changes

```bash
# Run all tests
cargo test --all

# Run clippy
cargo clippy

# Format code
cargo fmt

# Test a specific file
./target/release/zen your_test.zen
```

### 5. Submit a Pull Request

- Describe what the PR does and why
- Reference any related issues
- Ensure CI passes

## Code Style

### Rust Code

- Follow standard Rust conventions
- Use `cargo fmt` for formatting
- Address clippy warnings
- Document public APIs

### Zen Code (stdlib)

- Use 4-space indentation
- Document public functions
- Follow existing patterns in stdlib/

## Architecture Guidelines

### Compiler Pipeline

```
Source → Lexer → Parser → AST → TypeChecker → TypedAST → Codegen → LLVM IR → Binary
```

### Key Principles

1. **Single Source of Truth**: Type information lives in TypeChecker
2. **AST-First**: Work with parsed AST, not string manipulation
3. **Error Recovery**: Parser and typechecker should recover from errors
4. **LSP Integration**: Features should work in both CLI and LSP

### Adding a New Feature

1. **AST**: Add types to `src/ast/`
2. **Parser**: Add parsing in `src/parser/`
3. **TypeChecker**: Add type checking in `src/typechecker/`
4. **Codegen**: Add code generation in `src/codegen/llvm/`
5. **LSP**: Update LSP support in `src/lsp/`
6. **Tests**: Add tests in `tests/`
7. **Docs**: Update documentation

## Testing

### Test Categories

- **Unit Tests**: In-module `#[test]` functions
- **Integration Tests**: `tests/*.rs` files
- **Behavioral Tests**: `tests/behavioral_tests.rs` - compile and run Zen code
- **LSP Tests**: `tests/lsp/` - protocol tests

### Writing Tests

```rust
#[test]
fn test_your_feature() {
    // Test implementation
}
```

For behavioral tests that compile and run Zen code:

```rust
#[test]
fn test_zen_feature() {
    let code = r#"
        main = () {
            // Your Zen code
        }
    "#;

    let result = compile_and_run(code);
    assert!(result.is_ok());
}
```

## Documentation

- `docs/OVERVIEW.md` - Language syntax and features
- `docs/ARCHITECTURE.md` - Compiler internals
- `docs/INTRINSICS_REFERENCE.md` - Compiler intrinsics
- `docs/LSP_STATUS.md` - LSP capabilities

## Getting Help

- Read existing code for patterns
- Check `docs/` for architecture information
- Look at recent commits for examples

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.
