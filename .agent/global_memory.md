# Zen Language Global Memory

## Project Overview
The Zen programming language is a modern systems programming language designed for clarity, performance, and joy. It prioritizes explicit, consistent, and elegant syntax.

## Key Language Features (as per lang.md spec)

### Core Syntax Elements
1. **NO if/else keywords** - Only uses `?` operator for pattern matching
2. **Single loop construct** - `loop` keyword for all iteration
3. **Unified conditional operator** - `?` with `|` for patterns, `->` for destructuring, `=>` for results
4. **Consistent declarations** - `:=` immutable, `::=` mutable, with explicit type variants

### File Structure
- Extension: `.zen`
- Entry point: `main = () void { }`
- Comments: `// single line`
- Encoding: UTF-8

### Module System
- Bootstrap: `@std` namespace (`@std.core`, `@std.build`)
- Imports: Via `comptime` blocks using `build.import()`

### Type System
- Basic: `bool`, `void`, `string`, `i8-64`, `u8-64`, `f32/64`, `usize`
- Pointers: `Ptr<T>` (raw), `Ref<T>` (managed)
- Collections: Arrays `[N]T`, ranges `start..end` or `start..=end`
- Special: `type`, `Any`

### Pattern Matching Syntax
```zen
value ? | pattern => result
        | pattern -> binding => result
        | _ => default
```

### Error Handling
- `Result<T, E>` and `Option<T>` types
- No exceptions, errors as values
- Pattern matching for handling

### Advanced Features
- `comptime` for compile-time execution
- `async`/`await` for concurrency
- Behaviors (traits/interfaces)
- UFCS (Uniform Function Call Syntax)
- String interpolation with `$(expr)`

## Project Status
- Core parser: ✅ Complete
- Type system: ✅ Complete  
- Pattern matching: ✅ Complete (new `?` syntax)
- Error handling: ✅ Complete
- Compile-time: 🚧 In progress
- Async: 📋 Planned
- Standard library: 🚧 In progress

## File Organization
```
/home/ubuntu/zenlang/
├── src/           - Rust implementation
├── examples/      - Zen example files
├── zen_test/      - Test files
├── .agent/        - Agent meta information
├── lang.md        - Language specification
├── README.md      - Project documentation
└── ZEN_GUIDE.md   - Language guide
```

## Current Implementation Status
- Language name: Consistently "zen" throughout
- File extension: `.zen` everywhere
- Examples: 19+ example files demonstrating features
- Tests: 7 test files for validation
- Documentation: Complete specification in lang.md