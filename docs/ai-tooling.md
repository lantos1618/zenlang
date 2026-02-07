# Zen AI Tooling

CLI commands that expose the compiler's semantic analysis for AI coding assistants
(Claude Code, Cursor, Copilot, etc.) and other automated tools.

## Why not LSP?

LSP is a stateful bidirectional protocol designed for editors maintaining a live session.
AI tools want: "give me the answer to one question, then go away." These CLI commands
provide single-shot, structured output that any tool can consume.

## Commands

### `zen analyze <file> [--json]`

Full semantic analysis. Dumps everything the typechecker knows about a file:
functions, structs, enums, variables (with inferred types), methods, type aliases,
and behavior implementations. Uses error-tolerant mode — returns partial results
even when the file has type errors.

```bash
# Human-readable
zen analyze app.zen

# Structured JSON for AI tools
zen analyze app.zen --json
```

JSON schema:
```json
{
  "success": true,
  "file": "app.zen",
  "functions": {
    "main": {
      "params": [],
      "return_type": "i32",
      "is_external": false
    }
  },
  "structs": { "Point": { "fields": [{"name": "x", "type": "f64"}] } },
  "enums": {},
  "variables": { "main::count": "i32", "main::name": "StaticString" },
  "methods": {},
  "type_aliases": {},
  "behavior_impls": {},
  "constructors": []
}
```

### `zen check <file> [--json]`

Type-check a file and report **all** diagnostics. Runs the full compiler pipeline
(parse, import resolution, typechecking) and collects every error — not just the first.
Continues past declaration errors and body errors to report as many issues as possible.

```bash
# Human-readable
zen check app.zen

# Structured JSON for AI tools
zen check app.zen --json
```

JSON schema:
```json
{
  "success": false,
  "file": "app.zen",
  "diagnostics": [
    {
      "severity": "error",
      "code": "undeclared-variable",
      "message": "Undeclared variable: x",
      "line": 4, "column": 5,
      "end_line": 4, "end_column": 6
    },
    {
      "severity": "error",
      "code": "undeclared-variable",
      "message": "Undeclared variable: y",
      "line": 8, "column": 5,
      "end_line": 8, "end_column": 6
    },
    {
      "severity": "error",
      "code": "type-error",
      "message": "Cannot apply + to String and i32",
      "line": 12, "column": 5,
      "end_line": 12, "end_column": 10
    }
  ],
  "summary": { "errors": 3, "warnings": 0 }
}
```

### `zen query type <file>:<line>:<col>`

Point query: what is the type of the symbol at a specific position?
Supports member access — querying `field` in `obj.field` resolves the field's type
through the receiver's type. Also resolves UFC method calls.

```bash
# Simple variable
zen query type app.zen:15:5
# { "symbol": "count", "type": "i32", "kind": "variable", "scope": "main" }

# Member access (field)
zen query type app.zen:15:29
# { "symbol": "p.x", "type": "f64", "kind": "field", "receiver_type": "Point", "scope": "distance" }

# Member access (method)
zen query type app.zen:20:12
# { "symbol": "p.distance", "type": "(p: Point) f64", "kind": "method", "receiver_type": "Point" }
```

Possible `kind` values: `variable`, `parameter`, `function`, `struct`, `enum`,
`type_alias`, `module`, `field`, `method`, `function (UFC)`, `unknown`.

### `zen query methods <TypeName> <file>`

List all methods, fields, constructors, and UFC-compatible functions for a type.
Answers: "what can I do with this type?"

```bash
zen query methods Point app.zen
```

JSON schema:
```json
{
  "type": "Point",
  "fields": [
    { "name": "x", "type": "f64" },
    { "name": "y", "type": "f64" }
  ],
  "methods": [
    {
      "name": "distance",
      "kind": "function (UFC)",
      "params": [{"name": "p", "type": "Point"}],
      "return_type": "f64"
    },
    {
      "name": "new",
      "kind": "constructor",
      "params": [],
      "return_type": "Point"
    }
  ],
  "behaviors": ["Display"]
}
```

Method `kind` values: `method` (defined on type), `constructor`, `function (UFC)` (free function callable as method).

### `zen symbols <file> [--json]`

Lightweight symbol listing. Lists all declarations with their types and locations,
without running full type inference on function bodies. Faster than `analyze`
for getting an overview.

```bash
zen symbols app.zen --json
```

JSON schema:
```json
{
  "file": "app.zen",
  "symbols": [
    { "name": "Point", "kind": "struct", "line": 1, "fields": ["x: f64", "y: f64"] },
    { "name": "main", "kind": "function", "line": 10, "signature": "() i32" }
  ]
}
```

Possible `kind` values: `function`, `struct`, `enum`, `method`, `type_alias`,
`behavior`, `trait`, `constant`, `external_function`.

### `zen symbols --recursive <dir> [--json]`

List symbols across all `.zen` files in a directory tree. Indexes an entire project
in one call.

```bash
zen symbols --recursive src/ --json
```

JSON schema:
```json
{
  "directory": "src/",
  "files": [
    {
      "file": "src/main.zen",
      "symbols": [
        { "name": "main", "kind": "function", "line": 1, "signature": "() i32" }
      ]
    },
    {
      "file": "src/utils.zen",
      "symbols": [
        { "name": "Point", "kind": "struct", "line": 1, "fields": ["x: f64", "y: f64"] }
      ]
    }
  ],
  "total_files": 2,
  "total_symbols": 2
}
```

## Usage by AI Assistants

AI agents working with Zen code can use these commands to:

1. **Understand errors**: `zen check app.zen --json` gives ALL structured diagnostics
   in a single call — the agent can batch-fix multiple errors without round-trips.

2. **Understand types**: `zen analyze app.zen --json` reveals all inferred types,
   so the agent knows what type a variable is without guessing.

3. **Point queries**: `zen query type file:line:col` answers "what is this?"
   for any symbol, including member access like `obj.field`.

4. **Explore types**: `zen query methods TypeName file.zen` answers "what can I do
   with this type?" — lists fields, methods, constructors, and UFC functions.

5. **Navigate structure**: `zen symbols app.zen --json` gives a quick overview
   of what's defined where.

6. **Index projects**: `zen symbols --recursive src/ --json` indexes all declarations
   across an entire project in a single call.

## Diagnostic Error Codes

`zen check --json` returns structured error codes in the `code` field:

| Code | Meaning |
|------|---------|
| `syntax-error` | Invalid syntax |
| `parse-error` | Parser failure |
| `type-mismatch` | Incompatible types |
| `type-error` | General type error |
| `undeclared-variable` | Variable not in scope |
| `undeclared-function` | Function not found |
| `missing-return` | Function missing return |
| `duplicate-declaration` | Name declared twice |
| `import-error` | Module import failure |
| `unexpected-token` | Unexpected token in parsing |
| `invalid-pattern` | Bad pattern match |
| `comptime-error` | Compile-time evaluation error |
| `unsupported-feature` | Feature not yet implemented |
| `ffi-error` | FFI binding error |
| `cyclic-dependency` | Circular module dependency |
| `missing-type-annotation` | Type annotation required |
| `invalid-syntax` | Invalid syntax with suggestion |

## Design Principles

- **Single-shot**: Each command reads a file, does its work, prints output, exits.
  No persistent state, no protocol negotiation.

- **JSON first**: Every command supports `--json` for machine consumption.
  Human-readable output is the default for developer use.

- **Compiler-backed**: All type information comes from the real typechecker,
  not heuristics. If the compiler knows it, the CLI exposes it.

- **Error-tolerant**: Commands produce partial results when possible.
  `zen check` reports ALL errors, not just the first. `zen analyze` returns
  partial type context even when the file has errors. Declaration errors
  don't kill subsequent analysis.

- **All errors at once**: `zen check --json` collects every error the compiler
  finds — declaration errors, body type errors, everything. An AI agent
  can batch-fix multiple issues in one pass.

- **Project-wide**: `zen symbols --recursive` indexes entire directory trees,
  giving AI tools a complete project map in one call.
