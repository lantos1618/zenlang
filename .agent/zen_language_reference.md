# Zen Language Complete Reference

## Language Philosophy
- **Clarity over cleverness**: Code is read more often than written
- **Explicit over implicit**: No hidden control flow or magic
- **Minimal but composable**: Small set of powerful primitives
- **Errors as values**: No exceptions, use Result/Option types

## File Format
- Extension: `.zen`
- Encoding: UTF-8
- Comments: `// Single-line only`
- Entry point: `main = () void { }`

## Core Syntax Elements

### 1. Module System (@std Namespace)
```zen
comptime {
    core := @std.core        // Compiler intrinsics
    build := @std.build      // Build system interface
    io := build.import("io") // Import modules
}
```

### 2. Variable Declarations
| Syntax | Mutability | Type | Usage |
|--------|------------|------|-------|
| `x := value` | Immutable | Inferred | Primary constant |
| `x ::= value` | Mutable | Inferred | Primary variable |
| `x: T = value` | Immutable | Explicit | Typed constant |
| `x:: T = value` | Mutable | Explicit | Typed variable |
| `x:: T` | Mutable | Explicit | Default initialized |

### 3. Basic Types
- **Primitives**: `bool`, `void`, `string`
- **Integers**: `i8/16/32/64`, `u8/16/32/64`, `usize`
- **Floats**: `f32`, `f64`
- **Pointers**: `Ptr<T>` (raw), `Ref<T>` (managed)
- **Special**: `type` (type of types), `Any` (dynamic)

### 4. Data Structures

#### Structs (Product Types)
```zen
Person = {
    name: string,
    age: u32,
    score:: i32 = 0,  // Mutable with default
}
```

#### Enums (Sum Types)
```zen
Status =
    | Active
    | Inactive
    | Error({ code: i32, message: string })
```

### 5. Functions
```zen
// Basic function
add = (x: i32, y: i32) i32 {
    return x + y
}

// UFCS (Uniform Function Call Syntax)
area = (rect: Rectangle) f64 { ... }
my_rect.area()  // Can call as method
```

### 6. Pattern Matching (NO if/else keywords!)
```zen
// Simple matching
result := value ? | pattern1 => expr1
                 | pattern2 => expr2
                 | _ => default

// With destructuring using ->
status ? | .Error -> err => "Error: $(err.message)"
        | .Success -> data => "Data: $(data)"

// With guards
score ? | s -> s >= 90 => "A"
        | s -> s >= 80 => "B"
        | _ => "F"
```

### 7. Loops (Single 'loop' keyword)
```zen
// Conditional loop (while-like)
loop condition {
    // body
}

// Range loops
loop (0..10 ){ }      // Exclusive: 0-9
loop (0..=10) { }     // Inclusive: 0-10
```

### 8. Error Handling
```zen
// Result type
Result<T, E> = | Ok(T) | Err(E)

// Option type  
Option<T> = | Some(T) | None

// Pattern match to handle
result ? | .Ok -> value => use_value(value)
        | .Err -> error => handle_error(error)
```

### 9. Behaviors (Traits/Interfaces)
```zen
Drawable = behavior {
    draw = (self) void,
}

Circle.impl = {
    Drawable: {
        draw = (self: Circle) void { ... }
    }
}
```

### 10. Compile-Time (comptime)
```zen
// Compile-time block
TABLE := comptime {
    // Code executed at compile time
    generate_lookup_table()
}

// Generic functions
make_array = (comptime T: type, comptime N: usize) [N]T {
    // if comptime we can traverse
    // T.params, which is a list of args
    // T.body, which is the body ast
    // T.name, which is the name of the AST
    // T.type, struct, enum, function...

    return [N]T{}
}
```

### 11. String Interpolation
```zen
name := "Alice"
score := 95
io.print("User: $(name), Score: $(score)")
```

## Key Differences from Other Languages

1. **No if/else/switch**: Everything uses `?` operator
2. **No class keyword**: Use structs with UFCS
3. **No for/while**: Single `loop` construct
4. **No exceptions**: Errors as values (Result/Option)
5. **No implicit conversions**: Everything explicit
6. **No in**: Loops are only loop(...)
7. **Pattern matching everywhere**: Unified `?` operator

## Standard Library Modules

- `io`: Input/output operations
- `mem`: Memory management, allocators
- `math`: Mathematical functions
- `collections`: Data structures (List, Map, etc.)
- `string`: String utilities
- `fs`: File system operations
- `net`: Networking
- `async`: Async/await support

## Build System

The build system is accessed through `@std.build`:
- `build.import(module)`: Import a module
- Module resolution follows project structure
- Build configuration in build.zen files

## Memory Management

- Explicit allocator passing
- No hidden allocations
- RAII-style resource management
- Manual memory control with `Ptr<T>`
- Automatic management with `Ref<T>`

## Best Practices

1. **Use pattern matching**: Replace all conditional logic with `?`
2. **Prefer immutable**: Use `:=` by default, `::=` only when needed
3. **Handle errors explicitly**: Always match on Result/Option
4. **Leverage UFCS**: Write functions that work as methods
5. **Use comptime**: Move computation to compile time when possible
6. **Be explicit**: No implicit conversions or hidden behavior

## Common Patterns

### Error Propagation
```zen
process = () Result<Data, Error> {
    data := fetch_data() ? | .Ok -> d => d
                          | .Err -> e => return .Err(e)
    // Continue with data
}
```

### Resource Management
```zen
with_file = (path: string, fn: (File) void) Result<void, Error> {
    file := open_file(path) ? | .Ok -> f => f
                             | .Err -> e => return .Err(e)
    defer file.close()
    fn(file)
    return .Ok(void)
}
```

### Builder Pattern
```zen
Config = {
    host:: string = "localhost",
    port:: u16 = 8080,
    
    with_host = (self: Ref<Config>, host: string) Ref<Config> {
        self.host = host
        return self
    },
    
    with_port = (self: Ref<Config>, port: u16) Ref<Config> {
        self.port = port
        return self
    }
}

config := Config{}.with_host("example.com").with_port(443)
```