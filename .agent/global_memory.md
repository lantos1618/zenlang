# Zen Language Global Memory

## Project Overview
- **Language:** Zen - A modern systems programming language
- **Philosophy:** Clarity over cleverness, explicit over implicit, minimal but composable
- **Implementation:** Rust-based compiler with LLVM backend
- **Specification:** lang.md defines the language spec (v1.0 final cohesive draft)
- **Current State:** Core features implemented, fully aligned with lang.md spec

## Core Language Features (lang.md v1.0)

### Syntax Philosophy
- **NO traditional control flow keywords** (no `if`, `else`, `switch`, `match`)
- **Single unified operators** for common patterns
- **Explicit and consistent** declaration syntax

### Key Syntax Elements
1. **Declarations**:
   - `:=` - immutable binding (inferred type)
   - `::=` - mutable binding (inferred type)
   - `: Type =` - immutable with explicit type
   - `:: Type =` - mutable with explicit type

2. **Pattern Matching**: `scrutinee ? | pattern => expression`
   - Uses `->` for destructuring and binding
   - Replaces all conditional logic
   - Example: `score ? | 90..=100 => "A" | _ => "F"`

3. **Functions**: `name = (params) ReturnType { ... }`
   - UFCS support for method-like calls
   - Default parameters supported

4. **Data Structures**:
   - Structs: `TypeName = { field: Type, mutable_field:: Type }`
   - Enums: `TypeName = | Variant1 | Variant2(data)`

5. **Loops**: Single `loop` keyword for all iteration
   - Conditional: `loop condition { ... }`
   - Iterator: `loop item in collection { ... }`
   - Range: `loop i in start..end { ... }`

6. **Module System**:
   - `@std` namespace globally available
   - `@std.core` for intrinsics
   - `@std.build` for imports
   - Imports via `comptime { module := build.import("name") }`

7. **Error Handling**:
   - `Result<T, E>` and `Option<T>` types
   - No exceptions - errors as values
   - Pattern matching for error handling

8. **Compile-time**: `comptime` blocks and parameters
   - Metaprogramming
   - Generic functions
   - Compile-time computation

## Project Structure
- `/src` - Rust compiler implementation
  - `/lexer.rs` - Tokenization
  - `/parser.rs` - AST generation (aligned with spec)
  - `/typechecker/` - Type checking
  - `/codegen/` - LLVM code generation
- `/examples` - Example .zen files (comprehensive examples)
- `/tests` - Parser and feature tests
- `/.agent` - Project meta-information
- `/lang.md` - Language specification (source of truth)

## Implementation Status
✅ Parser fully aligned with lang.md spec
✅ All tests updated to match specification
✅ Comprehensive example (zen_comprehensive.zen) created
✅ Naming consistency verified (all "zen", no "glow")
✅ Documentation current

## Critical Reminders
- Entry point MUST be: `main = () void { ... }`
- File extension MUST be: `.zen`
- NO `if`/`else` keywords - use `?` operator
- Pattern matching uses `->` for destructuring
- Single `loop` keyword for ALL iteration