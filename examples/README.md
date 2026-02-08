# Zen Language Examples

Practical examples demonstrating Zen's core features.

## Running Examples

```bash
zen run examples/hello_world.zen
zen run examples/ffi_demo.zen
zen run examples/demo_project/main.zen
```

## Examples

### hello_world.zen
The classic first program - minimal and clear.

### ffi_demo.zen
Foreign Function Interface demonstration - calling C from Zen.

### demo_project/
Complete multi-file project demonstrating:
- Build system using `std.build`
- Standard library imports (`std.char`, `std.math`)
- Test framework using `std.testing.runner`
- Structs, enums, pattern matching, UFC
- Practical project organization

## Key Language Features

- **No Keywords**: No `if/else/while/for/match/class`
- **Pattern Matching**: Only `?` operator for all control flow
- **UFC**: Any function becomes a method
- **Expression-Based**: Everything returns a value
- **Type Safety**: Algebraic data types, no null
