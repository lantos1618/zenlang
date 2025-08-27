# Zen Language Complete Guide

## What Makes Zen Unique?

Zen is a modern systems programming language with a revolutionary approach to syntax. Unlike traditional languages that accumulate keywords over decades, Zen uses a minimal set of composable primitives.

### The "No Keywords" Philosophy

Most languages have 30-50+ keywords. Zen has just a handful:
- **No `if`, `else`, `elif`, `switch`, `match`** → Everything uses the `?` operator
- **No `fn`, `func`, `def`, `function`** → Functions use `=` syntax  
- **No `let`, `var`, `const`, `mut`** → Variables use `:=` family
- **No `for`, `while`, `do`** → Single `loop` keyword
- **No `class`, `impl`, `trait`** → Behaviors and implementations use simple blocks

## Core Syntax Rules

### 1. Pattern Matching is Everything (`?` operator)

```zen
// Traditional languages:
if x > 0 {
    return "positive"
} else if x < 0 {
    return "negative"  
} else {
    return "zero"
}

// Zen:
x ? | val -> val > 0 => "positive"
    | val -> val < 0 => "negative"
    | _ => "zero"
```

The `?` operator unifies:
- Simple conditionals
- Switch statements
- Pattern matching
- Destructuring
- Guard clauses

### 2. Variable Declaration Syntax

| Syntax | Mutability | Type | Use Case |
|--------|------------|------|----------|
| `name := value` | Immutable | Inferred | Most common - constants |
| `name ::= value` | **Mutable** | Inferred | Variables that change |
| `name: Type = value` | Immutable | Explicit | When type clarity needed |
| `name:: Type = value` | **Mutable** | Explicit | Typed variables |

Remember: Single `:` = immutable, Double `::` = mutable

### 3. Function Syntax

```zen
// Simple function
add = (a: i32, b: i32) i32 { a + b }

// With default parameters
greet = (name: string, prefix: string = "Hello") void {
    io.print("$(prefix), $(name)!")
}

// Generic function (compile-time)
make_array = (comptime T: type, comptime N: usize) [N]T {
    return [N]T{}
}
```

### 4. Loop Patterns

```zen
// Conditional (while-like)
loop condition {
    // body
}

// Infinite loop
loop {
    // body
    break  // Exit condition
}
```

## Data Structures

### Structs (Product Types)

```zen
Person = {
    name: string,
    age: int,
    email:: Option<string> = None,  // Mutable optional field
}

// Instantiation
alice := Person{ name: "Alice", age: 30 }
alice.email = Some("alice@example.com")
```

### Enums (Sum Types)

```zen
Result<T, E> = 
    | Ok(value: T)
    | Err(error: E)

// With anonymous structs
Message =
    | Text(content: string)
    | Image({ url: string, width: int, height: int })
    | Video({ url: string, duration: int })
```

## Advanced Pattern Matching

### Destructuring with `->`

The `->` operator is used for binding values in patterns:

```zen
// Bind and check
score ? | s -> s >= 90 => "A"
        | s -> s >= 80 => "B"
        | _ => "F"

// Destructure enums
result ? | .Ok -> value => process(value)
         | .Err -> error => handle_error(error)

// Destructure structs
point ? | { x -> xval, y -> yval } => "Point at $(xval), $(yval)"
```

### Complex Patterns

```zen
process_message = (msg: Message) void {
    msg ? | .Text -> content => io.print("Text: $(content)")
          | .Image -> { url, width, height } => {
              io.print("Image $(width)x$(height): $(url)")
          }
          | .Video -> data => {
              io.print("Video ($(data.duration)s): $(data.url)")
          }
}
```

## Error Handling

Zen uses explicit error values, no exceptions:

```zen
// Function that can fail
parse_int = (s: string) Result<int, string> {
    // Implementation...
}

// Handling errors
value := parse_int(input) ? | .Ok -> n => n
                           | .Err -> msg => {
                               log_error(msg)
                               0  // default value
                           }

// Propagating errors (conceptual)
process = () Result<Data, Error> {
    file_content := read_file("data.txt") ? | .Ok -> content => content
                                            | .Err -> e => return .Err(e)
    
    parsed := parse_data(file_content) ? | .Ok -> data => data
                                        | .Err -> e => return .Err(e)
    
    return .Ok(parsed)
}
```

## Compile-Time Programming

```zen
// Compile-time block
CONSTANTS := comptime {
    pi := 3.14159
    e := 2.71828
    { pi: pi, e: e }
}

// Compile-time function for generics
Container<T> = {
    items: []T,
    capacity: usize,
}

new_container = (comptime T: type, size: usize) Container<T> {
    return Container<T>{
        items: make_array(T, size),
        capacity: size,
    }
}

// Type-level programming
comptime {
    // This runs at compile time
    ValidTypes := [i32, f64, string]
    
    loop T in ValidTypes {
        // Generate specialized versions
    }
}
```

## Module System

```zen
// Every file starts with comptime imports
comptime {
    core := @std.core      // Compiler intrinsics
    build := @std.build    // Build system interface
    
    // Import modules
    io := build.import("io")
    net := build.import("net")
    json := build.import("json")
}

// Create namespace
Math = {
    PI := 3.14159,
    
    sin = (x: f64) f64 { /* ... */ },
    cos = (x: f64) f64 { /* ... */ },
}

// Use namespace
result := Math.sin(Math.PI / 2)
```

## UFCS (Uniform Function Call Syntax)

Any function can be called as a method:

```zen
// Define a type
Vector = { x: f64, y: f64 }

// Free functions
length = (v: Vector) f64 {
    return math.sqrt(v.x * v.x + v.y * v.y)
}

normalize = (v: Vector) Vector {
    len := v.length()  // UFCS call to length(v)
    return Vector{ x: v.x / len, y: v.y / len }
}

// Usage
vec := Vector{ x: 3.0, y: 4.0 }
len := vec.length()           // Same as length(vec)
normalized := vec.normalize() // Same as normalize(vec)
```

## Memory Management

```zen
// Pointer types
raw_ptr: Ptr<int>       // Raw unsafe pointer
managed: Ref<Data>      // Managed reference

// Allocator pattern
create_buffer = (allocator: Allocator, size: usize) []u8 {
    return allocator.alloc(u8, size)
}

// Defer for cleanup
process_file = (path: string) Result<void, Error> {
    file := open_file(path)?
    defer file.close()  // Runs at scope exit
    
    // Process file...
}
```

## Behaviors (Traits/Interfaces)

```zen
// Define behavior
Drawable = behavior {
    draw = (self, canvas: Canvas) void,
    bounds = (self) Rectangle,
}

// Implement for a type
Circle = { center: Point, radius: f64 }

Circle.impl = {
    Drawable: {
        draw = (self: Circle, canvas: Canvas) void {
            // Drawing logic
        },
        bounds = (self: Circle) Rectangle {
            // Return bounding box
        },
    }
}

// Generic function using behavior
render_all = (items: []Drawable, canvas: Canvas) void {
    loop item in items {
        item.draw(canvas)
    }
}
```

## String Interpolation

```zen
name := "Alice"
age := 30
score := 95.5

// Basic interpolation
message := "Hello $(name), you are $(age) years old"

// Expressions in interpolation
result := "Your grade: $(score > 90 ? "A" : "B")"

// Formatting (conceptual)
formatted := "Score: $(score:.2f)%"  // 95.50%
```

## Common Patterns

### Builder Pattern
```zen
Config = {
    host:: string = "localhost",
    port:: u16 = 8080,
    debug:: bool = false,
}

config := Config{}
    .with_host("example.com")
    .with_port(443)
    .with_debug(true)
```

### Result Chaining
```zen
process_data = (input: string) Result<Output, Error> {
    return parse(input)
        .and_then(validate)
        .and_then(transform)
        .map(format)
}
```

### State Machines
```zen
State = 
    | Idle
    | Processing(task: Task)
    | Complete(result: Result)
    | Error(message: string)

transition = (state: State, event: Event) State {
    return state ? 
        | .Idle -> event ? 
            | .Start -> task => .Processing(task)
            | _ => state
        | .Processing -> task -> event ?
            | .Complete -> result => .Complete(result)
            | .Error -> msg => .Error(msg)
            | _ => state
        | _ => state
}
```

## Best Practices

1. **Prefer pattern matching over nested conditions**
2. **Use destructuring to extract values cleanly**
3. **Make invalid states unrepresentable with enums**
4. **Use Result/Option for error handling, never panic**
5. **Leverage UFCS for method-like syntax**
6. **Keep functions small and composable**
7. **Use comptime for zero-cost abstractions**
8. **Document behaviors at the type level**

## Quick Reference Card

```zen
// Variables
x := 5           // immutable
y ::= 10         // mutable

// Functions  
f = (x: i32) i32 { x + 1 }

// Conditionals (no if!)
result := x ? | 0 => "zero"
             | _ => "non-zero"

// Loops
loop i in 0..10 { }      // for-like
loop x > 0 { x = x - 1 }  // while-like

// Structs
Point = { x: f64, y: f64 }

// Enums
Option<T> = | Some(T) | None

// Pattern matching with binding
value ? | .Some -> x => use(x)
       | .None => default()

// String interpolation
"Hello $(name), score: $(score)"

// Comptime
comptime { /* compile-time code */ }

// Module imports
comptime {
    io := @std.build.import("io")
}
```