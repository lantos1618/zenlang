#!/bin/bash
# Quarantined setup script for the gated Zen language-server surface.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Zen language-server setup is gated in the rewrite baseline."
echo "No tested server binary is defined by Cargo.toml."

# Build VS Code extension if needed
if [ -d "$SCRIPT_DIR/vscode-extension" ]; then
    echo "Building VS Code extension syntax/tooling support..."
    cd "$SCRIPT_DIR/vscode-extension"
    if [ ! -d "node_modules" ]; then
        npm install
    fi
    npm run compile
    echo "VS Code extension compiled"
    echo ""
    echo "To use in VS Code:"
    echo "   1. Open vscode-extension folder in VS Code"
    echo "   2. Press F5 to launch Extension Development Host"
fi
