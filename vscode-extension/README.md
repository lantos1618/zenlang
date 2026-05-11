# Zen Language Support for VS Code

This extension provides language support for the Zen programming language in Visual Studio Code.

## Features

- **Syntax Highlighting**: Full syntax highlighting for Zen code
- **CodeLens Actions**: Inline buttons to run and build functions
  - Run button for all functions
  - Build button for main and build entry points
  - Automatic detection of `main` and `build` functions
  - Click to execute directly from your editor
- **Code Snippets**: Common Zen code patterns

Language-server features are not part of the rewrite baseline. They are gated
until a tested server binary is added to the package.

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
3. CodeLens commands can call the local `zen` compiler where supported

### Using CodeLens Actions

1. The extension automatically detects functions in your code
2. Look for Run and Build buttons above function definitions
3. Click Run to execute a function
4. Click Build to compile a function (available for `main` and `build` functions)
5. Output appears in the "Zen Run" or "Zen Build" output channel at the bottom

See [CODELENS_FEATURE.md](./CODELENS_FEATURE.md) for detailed information about the CodeLens feature.

## Development

To work on this extension:

1. Open the `vscode-extension` folder in VS Code
2. Run `npm install` to install dependencies
3. Press F5 to launch the Extension Development Host
4. Make changes and reload the window to test

## Known Issues

- Language-server features are gated and not shipped in this rewrite package
- Performance may vary with large files

## Release Notes

### 0.1.0

Initial release with basic language support:
- Syntax highlighting
- CodeLens command integration
