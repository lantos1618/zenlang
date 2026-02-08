# Zen Demo Project

A complete Zen project demonstrating multi-file organization, build system, and testing.

## Files

- `build.zen` - Advanced build system using real stdlib with realistic dependencies
- `main.zen` - Complete showcase of all language features
- `test.zen` - Comprehensive test suite demonstrating testing patterns
- `utils.zen` - Utility library demonstrating static library creation

## Features Demonstrated

### Core Language Features
1. **Structs** - Custom data types with fields
2. **Enums** - Sum types with variants and payloads
3. **Functions** - Various function patterns including generics
4. **Closures** - Anonymous functions with captures
5. **Pattern Matching** - Exhaustive matching with `{}` syntax
6. **Error Propagation** - Using `.raise()` for Result/Option types
7. **Loops and Ranges** - Range loops `(0..10).loop()` and infinite loops
8. **Collections** - HashMap, DynVec, Array with allocators (NO-GC)
9. **Strings** - Static strings and string operations
10. **Generics** - Including nested generics like `Result<Option<T>,E>`
11. **UFC** - Universal Function Call for method chaining
12. **Forward Declarations** - Declaring functions before implementation
13. **Meta Programming** - Compile-time features like `@sizeof`
14. **FFI** - Foreign Function Interface for C interop
15. **Default Values** - Struct field defaults
16. **Memory Management** - Manual allocation with allocators

### Memory Model (NO-GC)
All collections in Zen REQUIRE explicit allocators:
```zen
allocator := get_default_allocator()
map := HashMap.new<K,V>(allocator)
vec := DynVec.new<T>(allocator)
arr := Array.new<T>(allocator)
```

### Pattern Matching
Zen uses `{}` syntax for pattern matching:
```zen
status ?
    | Active(id) { /* handle active */ }
    | Inactive { /* handle inactive */ }
    | Banned(reason, until) { /* handle banned */ }
```

### Error Handling
Result and Option types with `.raise()` for propagation:
```zen
x = divide(10.0, 2.0).raise()  // Propagates error if Err
y = find_person(1).raise()      // Propagates None as error
```

### Loop Syntax
Zen's loops manage internal state:
```zen
(0..10).loop((i) { println(i) })           // Range loop
loop((handle) { handle.break() })          // Infinite with break
collection.loop((item, index) { ... })     // Collection iteration
```

## Running the Example

```bash
# Compile and run main example
zen main.zen

# Or compile to executable
zen main.zen -o zen_demo
./zen_demo

# Run tests
zen test.zen
```

## Build System

The `build.zen` file demonstrates an advanced Zen build system with:
- **Real stdlib integration** using `std.build` and proper imports
- **Conditional compilation** based on `--release`/`--debug` flags
- **OS-specific configuration** with pattern matching for Linux/macOS/Windows
- **Realistic external dependencies** from GitHub (SDL2, ImGui, nlohmann-json)
- **Static library creation** for utility functions
- **FFI bindings** with system libraries (libc, libm, frameworks)
- **Symbol definition** for conditional compilation
- **Install configuration** with proper directory structure

### Usage
```bash
# Run the main example (which uses the build system)
zen main.zen

# Run tests
zen test.zen

# The build.zen file contains build configuration functions
# that are imported and used by main.zen
```

## Notes

This example is designed to be educational and demonstrate syntax. Some features like FFI and meta programming may not be fully implemented yet but show the intended syntax and design.