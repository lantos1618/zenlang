# Zen Language Support for VS Code

This rewrite-baseline extension ships editor basics for `.zen` files:

- **Syntax highlighting** through the TextMate grammar.
- **Command palette actions**: `Zen: Run Zen Function` and
  `Zen: Build Zen Function` call the local `zen` compiler for the active file.
- **Language configuration** for comments, brackets, indentation, and words.

Semantic diagnostics, hover, completion, formatting, inline actions, and server
settings are gated until the compiler exposes a tested `zen lsp` path backed by
the CLI parser, resolver, typechecker, build graph, and diagnostics.

## Use

Install dependencies, compile, then press F5 in VS Code:

```bash
cd vscode-extension
npm install
npm run compile
```

Open a `.zen` file for highlighting. The command palette runs write to the "Zen Run"
or "Zen Build" output channel. The `zen` compiler must be available in `PATH`.

## Release Notes

0.1.0: syntax highlighting and command palette integration.
