# Zen Language Documentation

## For Users

| Document | Description |
|----------|-------------|
| [OVERVIEW.md](OVERVIEW.md) | Language syntax and features |
| [QUICK_START.md](QUICK_START.md) | Getting started guide |
| [INTRINSICS_REFERENCE.md](INTRINSICS_REFERENCE.md) | Compiler intrinsics reference |

## For Contributors

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Compiler pipeline, modules, key concepts |
| [ROADMAP.md](ROADMAP.md) | Development roadmap and priorities |
| [LSP_STATUS.md](LSP_STATUS.md) | Language server features and module structure |
| [TECHNICAL_DEBT_AUDIT.md](TECHNICAL_DEBT_AUDIT.md) | Known issues and improvement plan |
| [design/](design/) | Design documents (stdlib, type system, pointers, etc.) |
| [reviews/](reviews/) | Code audit reports |

## Design Documents

| Document | Description |
|----------|-------------|
| [design/STDLIB_DESIGN.md](design/STDLIB_DESIGN.md) | Standard library API design |
| [design/SEPARATION_OF_CONCERNS.md](design/SEPARATION_OF_CONCERNS.md) | Three-layer architecture |
| [design/TYPE_SYSTEM_CLEANUP.md](design/TYPE_SYSTEM_CLEANUP.md) | Type system cleanup plan |
| [design/SAFE_TYPE_SYSTEM_DESIGN.md](design/SAFE_TYPE_SYSTEM_DESIGN.md) | Safe type system rationale |
| [design/SAFE_POINTERS_DESIGN.md](design/SAFE_POINTERS_DESIGN.md) | Ptr<T> design |
| [design/META_AST_CODEGEN.md](design/META_AST_CODEGEN.md) | Comptime architecture |
| [design/PRIMITIVES_VS_FEATURES.md](design/PRIMITIVES_VS_FEATURES.md) | Decision tree |

## Other

| Document | Description |
|----------|-------------|
| [ai-tooling.md](ai-tooling.md) | CLI commands reference |
| [ai-tooling-honest-review.md](ai-tooling-honest-review.md) | AI tooling editorial |
| [blog-post.md](blog-post.md) | Blog post / marketing |
| [allocator-redesign.md](allocator-redesign.md) | Unified allocator design |

## Quick Commands

```bash
cargo build --release      # Build compiler
cargo test --all           # Run tests
./target/release/zen FILE  # Run a .zen file
./target/release/zen-lsp   # Start LSP
```
