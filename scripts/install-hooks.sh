#!/bin/bash
# Install git hooks for Zenlang development

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "Installing git hooks..."

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
# Pre-commit hook for Zenlang
# Ensures code quality before commits

set -e

echo "Running pre-commit checks..."

# Check formatting
echo "  Checking formatting..."
cargo fmt --check || {
    echo "ERROR: Code is not formatted. Run 'cargo fmt' first."
    exit 1
}

# Run clippy
echo "  Running clippy..."
cargo clippy --quiet -- -D warnings || {
    echo "ERROR: Clippy found issues. Fix them before committing."
    exit 1
}

# Run unit tests (quick)
echo "  Running unit tests..."
cargo test --lib --quiet || {
    echo "ERROR: Unit tests failed."
    exit 1
}

echo "Pre-commit checks passed!"
EOF

chmod +x "$HOOKS_DIR/pre-commit"
echo "Pre-commit hook installed at $HOOKS_DIR/pre-commit"
echo "Done!"
