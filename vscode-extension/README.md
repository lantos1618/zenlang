# Zen Language Support for VS Code

This extension provides language support for the Zen programming language in Visual Studio Code.

## Features

- **Syntax highlighting**: TextMate grammar support for `.zen` files.
- **Command palette actions**: `Zen: Run Zen Function` and
  `Zen: Build Zen Function` call the local `zen` compiler for the active file.
- **Language configuration**: comments, brackets, indentation, and word
  patterns for editor basics.

Language server, semantic diagnostics, hover, completion, formatting, and inline
editor actions are not shipped in this rewrite package. They stay gated until a
tested `zen lsp` binary is backed by the CLI parser, resolver, typechecker,
build graph, and diagnostics.

## Requirements

- The Zen compiler must be installed and available in your PATH

## Installation

1. Install dependencies:
```bash
cd vscode-extension
npm install
```

2. Compile the extension:
```bash
npm run compile
```

3. In VS Code, press F5 to launch a new Extension Development Host window with the extension loaded.

## Extension Settings

This extension contributes the following settings:

- No stable language-server settings are exposed by the rewrite baseline.

## Usage

1. Open any `.zen` file in VS Code
2. You'll see syntax highlighting immediately
3. Use the command palette to run `Zen: Run Zen Function` or
   `Zen: Build Zen Function`

### Using Command Palette Actions

The commands use the active `.zen` file and write output to "Zen Run" or
"Zen Build" output channels.

## Development

To work on this extension:

1. Open the `vscode-extension` folder in VS Code
2. Run `npm install` to install dependencies
3. Press F5 to launch the Extension Development Host
4. Make changes and reload the window to test

## Known Issues

- Language server features are gated and not shipped in this rewrite package
- Performance may vary with large files

## Release Notes

### 0.1.0

Initial release with basic language support:
- Syntax highlighting
- Command palette integration
