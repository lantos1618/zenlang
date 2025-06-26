Excellent. Here is the complete, unified language overview incorporating all our refinements. The syntax is now consistent, with the elegant `->` operator for conditionals and clear rules for declarations and loops.

---

### **Zen Language Overview (Conceptual v1.0 - Final Cohesive Draft)**

**Table of Contents:**

1.  **Philosophy & General Notes**
2.  **Modules & Bootstrapping: The `@std` Namespace**
3.  **Basic Types & Literals**
4.  **Declarations: Variables & Constants**
5.  **Data Structures: Structs & Enums**
6.  **Namespacing & Code Organization**
7.  **Functions**
8.  **Control Flow: Conditionals & Loops**
9.  **Error Handling: Errors as Values**
10. **Behaviors (Traits & Interfaces)**
11. **Compile-Time Metaprogramming (`comptime`)**
12. **Asynchronous Programming (`async`/`await`)**
13. **Memory Management & Allocators**
14. **String Interpolation**
15. **Putting It All Together: A Main Program Example**

---

### 1. Philosophy & General Notes

Zen is a modern systems programming language designed for clarity, performance, and joy. It prioritizes explicit, consistent, and elegant syntax that composes into powerful patterns.

*   **Core Tenets:**
    *   **Clarity over cleverness:** Code is read more often than it is written.
    *   **Explicit is better than implicit:** The language should minimize hidden control flow, memory allocations, and "magic."
    *   **Minimal but composable syntax:** A small set of powerful keywords and operators combine to handle all use cases, reducing the language's surface area.
    *   **Powerful compile-time metaprogramming:** Enables high performance, reduces boilerplate, and provides deep introspection capabilities.
    *   **Errors as values:** Functions that can fail return explicit `Result` or `Option` types, eliminating exceptions for control flow.

*   **File Format & Entry Point:**
    *   **File Extension:** `.zen`
    *   **Encoding:** UTF-8
    *   **Comments:** `// Single-line comment.`
    *   **Entry Point:** The program entry point is a public function named `main`.
        ```zen
        main = () void {
            // Program starts here
        }
        ```

### 2. Modules & Bootstrapping: The `@std` Namespace

Zen provides a single, globally available "magic" namespace: `@std`. It is the bootstrap mechanism for accessing compiler intrinsics and the build system.

*   `@std.core`: A module containing fundamental compiler intrinsics for type information (`core.sizeOf`), memory operations, and other low-level primitives.
*   `@std.build`: A module for interacting with the build system, primarily used for importing other modules.

A conventional file preamble uses `comptime` to set up aliases for core modules:

```zen
comptime {
    core := @std.core
    build := @std.build

    // Import standard library modules using the build interface.
    // The string is a module specifier resolved by the build system.
    io := build.import("io")
    mem := build.import("mem")
    math := build.import("math")
    collections := build.import("collections")
}
```

### 3. Basic Types & Literals

*   **Primitive Types:** `bool` (`true`, `false`), `void`, `string`.
*   **Integers:** `int8`, `int16`, `int32`, `int64` (signed); `uint8`, `uint16`, `uint32`, `uint64` (unsigned); `usize` (pointer-sized unsigned).
*   **Floats:** `float32`, `float64`.
*   **Pointer Types:**
    *   `Ptr<T>`: A raw, unsafe pointer to type `T`. Null is `core.null_ptr<T>()`.
    *   `Ref<T>`: A managed reference to `T`. (Semantics like ARC or ownership are TBD).
*   **Collection Literals (Conceptual):**
    *   Arrays: `[1, 2, 3]` might infer `Array<int>` or a fixed-size array type.
*   **Special Types:**
    *   `type`: The type of a type, used in `comptime` for generics.
    *   `Any`: A type that can hold any value, with runtime type information.
*   **Range Expressions:**
    *   `start..end`: Exclusive end (e.g., `0..5` yields 0, 1, 2, 3, 4).
    *   `start..=end`: Inclusive end (e.g., `0..=5` yields 0, 1, 2, 3, 4, 5).

### 4. Declarations: Variables & Constants

Zen has a simple, consistent system for defining bindings. The `=` symbol is used for assignment and top-level definitions, while the `:=` family is used for local bindings.

| Syntax          | Mutability | Type     | Description                                |
| --------------- | ---------- | -------- | ------------------------------------------ |
| `name := value`   | Immutable  | Inferred | The primary way to declare a local constant. |
| `name ::= value`  | **Mutable**    | Inferred | The primary way to declare a local variable. |
| `name: T = value` | Immutable  | Explicit | Declares a constant with an explicit type.   |
| `name:: T = value`| **Mutable**    | Explicit | Declares a variable with an explicit type.   |

**Declaration without Initialization:**
Declaring a variable with `name:: Type` initializes it to the type's default value (0, `false`, `None`, etc.). It is a compile error if the type has no defined default.

```zen
// Immutable bindings (Constants)
PI := 3.14159
MAX_USERS: uint32 = 1000

// Mutable bindings (Variables)
request_counter ::= 0
active_connections:: uint16 = 0

// Assignment to a mutable variable
request_counter = request_counter + 1

// Default initialization
user_score:: int32         // Initialized to 0
current_user:: Option<User> // Initialized to Option.None
```

### 5. Data Structures: Structs & Enums

Top-level type definitions use the `TypeName = { ... }` or `TypeName = | ...` syntax.

#### Structs (Product Types)

Structs define custom data types composed of named fields. Use `{ ... }` for the definition. Fields are immutable by default and can be marked mutable with `::`.

```zen
// A struct definition. Trailing commas are allowed.
Person = {
    name: string,
    age: int,
    is_member:: bool = false,      // Mutable field with a default value
    last_login: Option<int> = None, // Optional field
}

// Instantiation (struct literal)
alice := Person{ name: "Alice", age: 30 }

// Field access and mutation
io.print("Name: $(alice.name)") // Access: "Alice"
alice.is_member = true           // Mutate
```

#### Enums (Sum Types / Tagged Unions)

Enums define a type that can be one of several distinct variants, separated by `|`. Variants can hold data (payloads), including anonymous structs.

```zen
// An enum definition.
Action =
    | Stop
    | Go
    | Wait(duration: int)
    | Error({ code: int, message: string })

// Instantiation
action1 := Action.Wait(5)
action2 := Action.Error({ code: 404, message: "Not found" })
```

### 6. Namespacing & Code Organization

*   **Nested Definitions:** Group related definitions within a block to create a simple namespace.

    ```zen
    Geometry = {
        PI := 3.1415926535,
        Point2D = { x: float64, y: float64 },

        distance = (p1: Point2D, p2: Point2D) float64 {
            dx := p1.x - p2.x
            dy := p1.y - p2.y
            return math.sqrt(dx*dx + dy*dy)
        },
    }

    // Usage
    origin := Geometry.Point2D{ x: 0.0, y: 0.0 }
    dist := Geometry.distance(origin, Geometry.Point2D{ x: 3.0, y: 4.0 })
    ```

*   **Modules:** The primary organizational tool. Each file is a module. Use `build.import("module_name")` to load a module, which acts as a namespace.

### 7. Functions

Functions are defined with `name = (parameters) returnType { ... }`.

```zen
// A simple function
calculate_area = (width: float64, height: float64) float64 {
    return width * height
}

// A procedure (returns void) with a default parameter value
print_greeting = (name: string, prefix: string = "Hello") void {
    io.print("$(prefix), $(name)!\n")
}
```

**Uniform Function Call Syntax (UFCS):**
A free function whose first parameter is of type `T` can be called as if it were a method on an instance of `T`.

```zen
Rectangle = { width: float64, height: float64 }

// A free function associated with Rectangle
area = (rect: Rectangle) float64 {
    return rect.width * rect.height
}

my_rect := Rectangle{ width: 10.0, height: 5.0 }

area1 := area(my_rect)        // Standard call
area2 := my_rect.area()       // UFCS call, more idiomatic
```

### 8. Control Flow: Conditionals & Loops

#### Conditional Expression

Zen uses a single, unified construct for all conditional logic and pattern matching. It replaces `if-else` chains and `switch`/`match` statements with a more powerful and expressive syntax. The core structure uses the `?` operator for pattern matching: `scrutinee ? | pattern => expression`.

**Pattern Matching with `?`**

The `?` operator indicates pattern matching on a value. The scrutinee is available directly in the match arms without needing a capture variable.

**Example 1: Simple value matching**

```zen
score: int = 85

// Match directly on the score value
grade := score ? | 90..=100 => "A"
                 | 80..=89  => "B"
                 | 70..=79  => "C"
                 | _        => "D or F"
```

**Example 2: Enum destructuring with `->`**

```zen
// Use `->` for destructuring and binding values from enum variants
handle_action := action ? | .Error -> err => "Error: $(err.message)"
                         | .Wait -> duration => "Wait $(duration)ms"
                         | .Stop => "Stopping"
                         | _ => "Default action"
```

**Example 3: Struct destructuring**

```zen
point := Point{ x: 10, y: 20 }
description := point ? | { x -> x_val, y -> y_val } => "Point at $(x_val), $(y_val)"
```

**Example 4: With guards using `->`**

```zen
// Use `->` to bind a value and apply a guard condition
score ? | s -> s >= 90 => "A"
        | s -> s >= 80 => "B"
        | s -> s >= 70 => "C"
        | _ => "F"
```

**Example 5: Complex patterns with blocks**

```zen
result ? | .Ok -> value => {
            io.print("Success: $(value)")
            value
         }
         | .Err -> err => {
            io.print("Error: $(err)")
            0
         }
```

This creates an extremely clean, readable, and keyword-free system where:
- `?` indicates pattern matching
- `->` is used for destructuring and binding in patterns
- `=>` separates patterns from their result expressions
- Blocks `{ }` are optional for simple expressions

#### The `loop` Construct

`loop` is the only looping keyword, used for all iteration patterns.

*   **Conditional Loop (`while`-like):**

    ```zen
    counter ::= 10
    loop counter > 0 {
        io.print("$(counter)...")
        counter = counter - 1
    }
    io.print("Liftoff!")
    ```

*   **Iterable Loop (`for`-each like):**

    ```zen
    names := ["Alice", "Bob", "Charlie"]
    loop name in names {
        io.print("Hello, $(name)!")
    }
    ```

*   **Loop Control:** `break` exits a loop, and `continue` skips to the next iteration. Labels can be used for nested loop control: `my_label: loop { ... break my_label ... }`.

### 9. Error Handling: Errors as Values

Zen uses `Result<T, E>` and `Option<T>` enums for explicit, value-based error handling.

*   `Option<T>`: `| Some(T) | None`. For values that might be absent.
*   `Result<T, E>`: `| Ok(T) | Err(E)`. For operations that can fail with a specific error type.

The conditional expression is the primary tool for handling these types.

```zen
// A function that can fail
parse_int = (s: string) Result<int, ParseError> {
    // ... parsing logic ...
}

// Handling the result
parsed_value := parse_int("123a") ? | .Ok -> value => {
        io.print("Success: $(value)")
        value // This block evaluates to `value`
    }
    | .Err -> err => {
        io.print("Error: $(err.message)")
        0 // Return a default value on error
    }
```

### 10. Behaviors (Traits & Interfaces)

Behaviors define contracts (a set of method signatures) that types can implement, enabling polymorphism.

```zen
// Define a behavior
Writer = behavior {
    write = (self, data: []byte) Result<usize, io.Error>,
}

// Implement the behavior for a type
File = { fd: int, ... }

File.impl = { // Implementation block
    Writer: { // Implement the Writer behavior
        write = (self: Ptr<File>, data: []byte) Result<usize, io.Error> {
            // ... os call to write to self.fd ...
        }
    }
}

// A function that uses the behavior for static or dynamic dispatch
log_message = (output: Writer, message: string) void {
    // This function can write to a File, a network socket, a buffer, etc.
    _ = output.write(message.to_bytes())
}
```

### 11. Compile-Time Metaprogramming (`comptime`)

The `comptime` keyword designates code to be executed at compile time.

*   **`comptime` Blocks:** A block of code executed by the compiler.
    ```zen
    LOOKUP_TABLE := comptime {
        table:: [256]int
        loop i in 0..256 {
            table[i] = i * i
        }
        table // The block evaluates to this value
    }
    ```
*   **`comptime` Parameters:** For creating generic functions.
    ```zen
    // A generic function that takes a compile-time type `T`
    // and a compile-time value `N`.
    make_array = (comptime T: type, comptime N: usize) [N]T {
        return [N]T{} // Returns a default-initialized array
    }

    my_array := make_array(int32, 1024) // Creates a [1024]int32
    ```

### 12. Asynchronous Programming (`async`/`await`)

*   An `async` function returns a `Task<T>` instead of `T`. A `Task<T>` is a future or promise representing a computation that will eventually complete.
*   The `await` keyword suspends the execution of an `async` function until the `Task` it is waiting on completes.
*   An event loop (provided by the runtime or standard library) is responsible for scheduling and running tasks.

### 13. Memory Management & Allocators

Zen aims to give developers explicit control over memory.

*   **Pointers:** `Ptr<T>` for raw, unsafe memory access. `Ref<T>` for managed memory (details TBD).
*   **Allocators:** Memory allocation is handled via an `Allocator` interface (a behavior). Functions that need to allocate memory take an allocator as a parameter.
    ```zen
    // A function that needs dynamic memory
    create_list = (allocator: mem.Allocator, initial_items: []string) collections.List<string> {
        list := collections.List<string>.new(allocator)
        loop item in initial_items {
            list.append(item)
        }
        return list
    }
    ```

### 14. String Interpolation

Strings support interpolation using the `$(expression)` syntax for embedding values directly.

```zen
user := "Alice"
score := 95
io.print("User: $(user), Score: $(score)")
// Output: User: Alice, Score: 95
```

### 15. Putting It All Together: A Main Program Example

```zen
// main.zen

// 1. Imports and setup
comptime {
    build := @std.build
    io := build.import("io")
    str_utils := build.import("string")
}

// 2. Data structures
HttpError =
    | NotFound
    | BadRequest(reason: string)

// 3. Functions
handle_request = (url: string) Result<string, HttpError> {
    // Using the conditional expression with the new ? syntax
    return url ? | u -> str_utils.equals(u, "/home")      => .Ok("Welcome home!")
                | u -> str_utils.equals(u, "/users")     => .Ok("List of users")
                | u -> str_utils.starts_with(u, "/admin") => .Err(.BadRequest("Admin access required"))
                | _ => .Err(.NotFound)
}

// 4. Entry Point
main = () void {
    urls_to_test := ["/home", "/admin/dashboard", "/about"]

    loop url in urls_to_test {
        response_text := handle_request(url) ? | .Ok -> body => "SUCCESS: Fetched $(url) -> $(body)"
                                              | .Err -> .NotFound => "FAILURE: Path $(url) not found"
                                              | .Err -> .BadRequest(reason) => "FAILURE: Bad request for $(url): $(reason)"

        io.print_line(response_text)
    }
}
```