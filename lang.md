Yes, excellent! "Errors as values" is a modern and robust approach, aligning perfectly with Zen's explicit and controlled philosophy, and avoids the complexities and non-local control flow of try/catch exceptions.

This means functions that can fail will typically return a type like `Result<ValueType, ErrorType>` or `Option<ValueType>` (if the error is simply "not found" or "no value").

Let's create the Table of Contents first to structure the final draft, then proceed with the content in parts.

---

**Zen Language Overview (Conceptual v1.0 - Final Draft Outline)**

**Table of Contents:**

1.  **Philosophy & General Notes**
    *   Core Tenets
    *   File Format, Comments, Entry Point
    *   Core Language Keywords (Brief List)

2.  **Modules & Bootstrapping: `@std` Namespace**
    *   The `@std` Global Namespace (`@std.core`, `@std.build`)
    *   Typical File Preamble for Imports

3.  **Basic Types & Literals**
    *   Primitive Types (integers, floats, bool, string, void)
    *   Pointer Types (`Ptr<T>`, `Ref<T>`)
    *   Collection Types (Conceptual: `Array<T>`, `Option<T>`)
    *   Special Types (`Any`, `type`)
    *   Range Expressions
    *   `Task<T>` (for Async)

4.  **Declarations, Variables & Mutability**
    *   Binding Syntax (`:=`, `::=`, `:`, `::`)
    *   Declaration without Initialization (Default Values)

5.  **Data Structures**
    *   **Structs (Product Types)**
        *   Definition
        *   Instantiation (Struct Literals)
        *   Field Access
    *   **Enums (Sum Types / Tagged Unions)**
        *   Definition (Variants with Optional Data)
        *   Instantiation

6.  **Namespacing & Organization**
    *   Nested Definition Blocks (Simple Namespacing)
    *   Modules (Primary Organization via `@std.build.import`)

7.  **Functions**
    *   Definition Syntax (`name = (params) retType { body }`)
    *   Uniform Function Call Syntax (UFCS)
    *   Default Parameter Values

8.  **Control Flow**
    *   **Conditional Expression (Unified If/Match): `(ScrutineeOrTrue) { Arms... }`**
        *   General Conditional Logic (Implicit `true` Scrutinee)
        *   Matching on a Value
        *   Pattern Matching Capabilities (Type Checks, Literals, Guards - Conceptual)
        *   Exhaustiveness
    *   **Loop Construct: `loop`**
        *   Conditional Loop (`while`-like)
        *   Iterable Loop (Ranges, Collections)
        *   Loop Control (`break`, `continue`, Labeled Loops)
        *   Iteration Protocol (Conceptual)

9.  **Error Handling (Errors as Values)**
    *   `Result<ValueType, ErrorType>` Enum (Conceptual)
    *   Functions Returning `Result` or `Option`
    *   Handling Errors with Conditional Expressions

10. **Behaviors (Trait-like Contracts)**
    *   Definition (`BehaviorName: behavior { methods... }`)
    *   Implementation (e.g., `StructName: { BehaviorName: { methods... } }` section)
    *   Using Behaviors (Generics with Constraints, Dynamic Dispatch - Conceptual)

11. **Compile-Time Metaprogramming (`comptime`)**
    *   `comptime` Keyword (Blocks, Parameter Qualifiers)
    *   Accessing Core Intrinsics (via `core := @std.core`)
    *   Comptime Evaluation and Code Generation (Conceptual)

12. **Asynchronous Programming (`async`/`await`)**
    *   `async` Functions and `Task<T>`
    *   `await` Expression
    *   Event Loop / Scheduler (Runtime Responsibility)

13. **Memory Management & Allocators**
    *   `Ptr<T>` (Unsafe Pointers)
    *   `Ref<T>` (Managed References - Semantics TBD)
    *   Allocator Convention (Structs implementing an expected interface via duck typing)
    *   `mem := build.import("mem")` for standard allocators/types.

14. **String Interpolation**

15. **Main Program Example & Standard Library Structure (Conceptual)**

---

This looks like a comprehensive structure. I will now generate the content in multiple parts, starting with sections 1-4.

**(1/N - Will indicate when to prompt for next part)**

Okay, here's the first part of the Zen Language Overview v1.0, covering sections 1-4 from the Table of Contents.

(1/4)

// Zen Language Overview (Conceptual v1.0 - Final Draft)

// --- 1. Philosophy & General Notes ---

// **Core Tenets:**
// Zen aims to be a modern, explicit, and performant systems programming language
// with powerful compile-time metaprogramming capabilities. It prioritizes clarity,
// consistency, and developer control. The design features a minimalist core syntax
// that composes into expressive patterns, striving to reduce syntactic noise by
// eliminating redundant keywords and relying on structure and context. Error handling
// is done through value-based returns, avoiding exceptions for control flow.

// **File Format, Comments, Entry Point:**
// - File Encoding: UTF-8 assumed.
// - File Extension: .ln (suggestion)
// - Comments: // Single-line comment. (Block comments TBD, if needed)
// - Entry Point: A `main` function is the default program entry point.
//   `main = () void { ... }` or `main = async () Task<void> { ... }`

// **Core Language Keywords (Non-exhaustive, illustrative list):**
// comptime, loop, behavior, enum, async, await, return, break, continue.
// Built-in type names (string, int32, bool, void, Ptr, Ref, Array, Option, Any, type, etc.)
// are identifiers, not strictly keywords in all contexts, but have special meaning to the compiler.
// (Note: Mutability indicators like '::' and '::=' are operators/syntax.
//  Conditional logic uses a structural expression, not 'if' or 'match' keywords.)


// --- 2. Modules & Bootstrapping: `@std` Namespace ---

// **The `@std` Global Namespace:**
// - '@std' is the ONLY globally available "magic" namespace, provided by the compiler.
// - It's accessible in all contexts but primarily used in `comptime` blocks for setup.
// - It provides access to:
//     1. `@std.core`: A module object containing fundamental compiler intrinsics.
//        (e.g., for memory operations, type info, basic comptime control).
//     2. `@std.build`: A module object for build system interaction and importing other modules.
//        (e.g., for loading standard library components, project files, or packages).

// **Typical File Preamble (Comptime Setup):**
// These aliases are conventionally defined at the top of a file within a comptime block
// or directly if the top-level scope is implicitly comptime for declarations.
// For simplicity, direct aliasing shown here:
core := @std.core          // Alias for core compiler intrinsics
build := @std.build         // Alias for build system interface

// Import standard library modules (or project/package modules) using the build interface.
// The string argument to `build.import` is a module specifier, resolved by the build system.
io := build.import("io")                 // For I/O operations (streams, printing)
os := build.import("os")                 // For OS interactions (files, environment, exit)
str_utils := build.import("string")      // For string manipulation & utilities (named to avoid conflict)
math := build.import("math")             // For math functions and constants
collections := build.import("collections")  // For data structures (List, Map, etc.)
ct_utils := build.import("comptime_utils") // For higher-level comptime utilities
mem := build.import("mem")               // For memory management types (Allocator, default allocators)
async_mod := build.import("async")       // For Task<T>, async primitives (named to avoid conflict)
log_mod := build.import("log")           // For logging framework


// --- 3. Basic Types & Literals ---

// **Primitive Types (many may be provided by `core` or `build.import("types")`):**
// - Integers: int8, int16, int32, int64 (signed)
//             uint8, uint16, uint32, uint64 (unsigned)
//             usize (pointer-sized unsigned integer, for sizes and indices)
// - Floats:   float32, float64
// - Boolean:  bool (literals: `true`, `false`)
// - String:   string (UTF-8 encoded. Literals: `"hello"`, supports interpolation: `"val: $(expr)"`)
// - Void:     void (Represents no value, used as return type for procedures)

// **Pointer Types:**
// - `Ptr<T>`: Raw, unsafe pointer to type T.
//   - Null pointer: `core.nullPtr<T>()` or a dedicated literal like `null_ptr<T>`.
//   - Dereference: `ptr_val^` (syntax TBD, `^` is a placeholder).
// - `Ref<T>`: Managed reference/pointer to T. Semantics (e.g., ARC, ORC, ownership-based,
//   GC-integration) are a critical part of Zen's memory model, to be detailed.

// **Collection Types (Conceptual - likely from `collections` module):**
// - `Array<T>`: Fixed-size or dynamically-sized contiguous sequence of T.
//   - Literals (if supported): `Array<int32>.new(1, 2, 3)` or `[1, 2, 3]` (syntax TBD).
// - `Option<T>`: Represents an optional value. Defined as an enum:
//   `Option<T>: enum { Some(T), None }`
//   Used extensively for functions that might not return a value (instead of null pointers).

// **Special Types:**
// - `Any`: A hypothetical top type or type-erased container, used for dynamic type checks
//   in conditional expressions. Its exact nature and safety need careful definition.
// - `type`: A special type representing a type itself. Used in `comptime` for generics
//   and type manipulation. `Int32Type := int32 as type`.

// **Range Expressions (used with `loop` or for slicing):**
// - `start..end`: Exclusive end (e.g., `0..5` yields 0, 1, 2, 3, 4).
// - `start..=end`: Inclusive end (e.g., `0..=5` yields 0, 1, 2, 3, 4, 5).

// **Task Type (for Async - from `async_mod`):**
// - `Task<T>`: Represents the future result of an asynchronous operation yielding a `T`.
//   If an async operation doesn't yield a value, it returns `Task<void>`.


// --- 4. Declarations, Variables & Mutability ---

// **Binding Syntax:**
// - Immutable binding, type inferred: `name := expression`
//   `PI := 3.14159`
//   `Greeting := "Hello, Zen!"`
// - Mutable binding, type inferred: `name ::= expression`
//   `requestCounter ::= 0`
//   `currentMessage ::= str_utils.Builder.new()`
// - Immutable binding, explicit type: `name : type = expression`
//   `MaxUsers : uint32 = 1000`
//   `AppName : string = "Zen Central"`
// - Mutable binding, explicit type: `name :: type = expression`
//   `activeConnections :: uint16 = 0`
//   `buffer :: Array<byte> = collections.ArrayList<byte>.withCapacity(1024)`

// **Declaration without Initialization (Default Values):**
// - `name : type` or `name :: type`
// - The variable is initialized with the default value for its type.
// - For structs, this means all fields are initialized to their respective defaults.
// - For primitive types: 0 for numbers, false for bool, empty string for string,
//   null_ptr for Ptr<T>, Option.None for Option<T>.
// - It's an error to declare without initialization if a type has no defined default
//   (e.g., a struct field without a default value that isn't an Option<T>).

// Example:
defaultUserScore : uint32         // Initialized to 0
mutablePoint :: Point             // Initialized to Point{x:0.0, y:0.0} (if Point has defaults)
optionalName : Option<string>     // Initialized to Option<string>.None


(End of 1/4)

This covers the foundational elements. Ready for the next part (Data Structures, Namespacing, Functions, Conditional Expressions, Loops)?


Okay, here's the second part of the Zen Language Overview v1.0, covering Data Structures, Namespacing, Functions, the unified Conditional Expression, and the Loop construct.

(2/4)

// --- 5. Data Structures ---

// **Structs (Product Types):**
// - Define custom data types composed of named fields.
// - Definition:
//   `Person = { name: string, age: int, is_member: bool }`
// Example:
Person = {
    name: string,
    age: int,
    is_member: bool,
}

// - Instantiation (using struct literal syntax):
//   `alice := Person { name: "Alice", age: 30, is_member: true }`
// - Field access: `alice.name`, `alice.age`

// **Enums (Sum Types / Tagged Unions):**
// - Define a type that can be one of several distinct variants.
// - Each variant can optionally hold data of a specific type.
// - Definition:
//   `Color = | Red | Green | Blue`
//   `Action = | Stop | Go | Wait(duration: int)`
// Example:
Color =
| Red
| Green
| Blue

Action =
| Stop
| Go
| Wait(duration: int)

// - Instantiation:
//   `c := Color.Red`
//   `a := Action.Wait(5)`
// - Pattern matching uses the same `|` separator for arms (see Conditional Expression section).

// This style is minimal, elegant, and visually distinctive: use `{ ... }` for structs and `|` for enums.


// --- 6. Namespacing & Organization ---

// **Nested Definition Blocks (Simple Namespacing):**
// - A block `{...}` can group related definitions (types, constants, functions).
// - This provides a basic level of namespacing.
// Example:
Geometry: {
    PI : float64 = 3.1415926535, // Constant within Geometry
    Point2D: { x: float64, y: float64 }, // Struct within Geometry

    distance = (p1: Point2D, p2: Point2D) float64 {
        dx := p1.x - p2.x
        dy := p.y - p2.y // Typo: should be p1.y - p2.y
        return math.sqrt(dx*dx + dy*dy) // Assuming 'math' module imported
    }, // Comma if more items in Geometry block
}
// Usage:
// origin_geom := Geometry.Point2D{x:0.0, y:0.0}
// dist := Geometry.distance(origin_geom, Geometry.Point2D{x:3.0, y:4.0})

// **Modules (Primary Organization via `@std.build.import`):**
// - Zen code is organized into modules. Each file can be a module, or a directory
//   can represent a larger module with submodules.
// - The `build := @std.build` object's `import("specifier")` function loads modules.
// - Loaded modules are typically assigned to a variable, which then acts as a namespace.
//   `collections := build.import("collections")`
//   `my_list := collections.List<int32>.new()`
// - Visibility (public/private) of definitions within modules is TBD (e.g., `public` keyword).


// --- 7. Functions ---

// **Definition Syntax:**
// `funcName = (param1: type, param2: type = defaultValue) returnType { body }`
// - `returnType` is `void` if the function does not return a value.
// - Parameters with default values must come after parameters without defaults.
// Example:
calculateArea = (width: float64, height: float64) float64 {
    return width * height;
}

printGreeting = (name: string, prefix: string = "Hello") void {
    io.stdout.writeString("$(prefix), $(name)!\n") // Assuming 'io' module imported
}

// **Uniform Function Call Syntax (UFCS):**
// - If a free function's first parameter is of type `T` (or `Ptr<T>`, `Ref<T>`),
//   it can be called as if it were a method on an instance of `T`.
//   `instance_of_T.funcName(other_args)` is syntactic sugar for
//   `funcName(instance_of_T, other_args)`.
// Example:
Rectangle: { width: float64, height: float64 } // Struct
// Free function, conceptually an associated function for Rectangle:
Rectangle_area = (rect: Rectangle) float64 {
    return rect.width * rect.height;
}
my_rect := Rectangle{width: 10.0, height: 5.0}
area1 := Rectangle_area(my_rect)  // Standard call
area2 := my_rect.Rectangle_area() // UFCS call (no other args)


// --- 8. Control Flow ---

// **Conditional Expression (Unified If/Match - NO 'if' or 'match' KEYWORDS):**
// - Syntax: `scrutinee ? (pattern) => logic => result` or simply `scrutinee (pattern) => logic => result` with arms separated by `?`, `|`, or newlines.
// - This is the sole construct for conditional logic and pattern matching.
// - The scrutinee is the value to be matched. If omitted, it defaults to `true` (for boolean chains).
// - Each arm consists of:
//     - A pattern (with optional capture/destructuring)
//     - An optional logic expression (boolean, using `&&`, `||`, etc.)
//     - The result expression (evaluated if the pattern and logic match)
// - The construct is an expression; it evaluates to the result of the first matching arm.
// - Must be exhaustive if its value is used (i.e., all possible cases of the scrutinee must be covered by patterns, or a wildcard `_` pattern must be present).
//   If used as a statement (value discarded), non-exhaustive forms might be allowed.
//
// - **Pattern Matching Capabilities (Conceptual):**
//   - Literals: `10`, `"text"`, `true`, `HttpVerb.Get`
//   - Variables: `(x)` (binds the value to `x`)
//   - Wildcard: `_` (matches anything without binding)
//   - Enum Variant Checks & Destructuring: `(.Variant)`, `(.Variant(x))`, `(.Variant(x, y))`
//   - Logic: Any boolean expression after the capture, e.g. `(x) => x > 0 && x < 50`
//
// Example: General Conditional Logic (no 'if' keyword)
score
| score < 0                => "Invalid (negative)"
| score == 0               => "Zero"
| score > 0 && score < 50  => "Low"
| score >= 50 && score < 80 => "Medium"
| score >= 80 && score <= 100 => "High"
| _                        => "Unknown"

// Example: Matching on an Enum Value with Destructuring and Logic
result
| .Ok(x)                    => "Success: $(x)"
| .Err(code, msg) => code < 100   => "Minor error: $(msg)"
| .Err(code, msg) => code >= 100  => "Major error: $(msg)"
| _                              => "Unknown result"

// Example: Matching a struct with logic
person
| (p) => p.age < 18                   => "Minor: No access"
| (p) => p.age >= 18 && p.is_member   => "Adult member: Full access"
| (p) => p.age >= 18 && !p.is_member  => "Adult non-member: Limited access"
| _                                   => "Unknown"

// - Arms can be separated by `?`, `|`, or newlines for flexibility and readability.
// - No 'if', 'where', or other guard keywords are needed; logic is just a boolean expression after the capture.
// - Destructuring, logic, and result are all part of the arm, making the syntax minimal and expressive.


// **Loop Construct: `loop`**
// - Unified loop construct for all iteration and looping needs.
// - Loop Control: `break;` (exits innermost loop), `continue;` (skips to next iteration).
//   `break value;` can allow a loop to evaluate to a value (TBD).
// - Labeled Loops: `myLabel: loop (...) { ... break myLabel; ... }` for explicit control.

// - **Conditional Loop (`while`-like):**
//   `loop(boolean_condition) { _ => { /* body if condition true */ } }`
//   The `_ =>` arm executes if the condition (re-evaluated each iteration) is true.
// Example:
printCountdown = (from_val: int32, io_h: io.Stream) void {
    counter ::= from_val
    countdown_l: loop(counter >= 0) {
        _ => {
            io_h.writeString("$(counter)\n")
            counter = counter - 1
        }
    }
}

// - **Iterable Loop (Ranges, Collections):**
//   `loop(iterable_expression) { item_var => { /* for item */ } else => { /* if no break */ } }`
//   The `else => {}` block is optional and executes if the loop completes all iterations
//   without an explicit `break`.
// Example (Range):
sumRange = (start: int32, end_inclusive: int32) int32 {
    current_sum ::= 0
    range_l: loop(start..=end_inclusive) {
        i => { current_sum = current_sum + i }
        // No 'else' block in this example
    }
    return current_sum
}

// Example (Collection - assuming collections.List is iterable or provides .iter()):
logItems = (items: collections.List<string>, log_h: /* log_mod */) void {
    items_l: loop(items.iter()) { // .iter() assumed to return an iterator
        item => { log_h.print(log_h.Level.Info, "Item: $(item)") }
        else => { log_h.print(log_h.Level.Debug, "All items logged.") }
    }
}

// - **Iteration Protocol (Conceptual):**
//   For a type to be usable in an iterable loop, it must conform to an iteration protocol.
//   This likely involves implementing `Iterable` and `Iterator` behaviors (see Behaviors section).
//   `Iterable.iter()` would return an `Iterator`.
//   `Iterator.next()` would return `Option<ItemType>`.

// - **C-style For Loop (TBD):**
//   A C-style for loop syntax like `loop (init_stmt; condition_expr; post_iter_stmt) { body }`
//   could be another accepted form for the `loop()` construct if desired.


(End of 2/4)

Next up: Error Handling, Behaviors, and Comptime Metaprogramming. Ready for part 3?

Okay, here's the third part of the Zen Language Overview v1.0, covering Error Handling (Errors as Values), Behaviors, and Compile-Time Metaprogramming.

(3/4)

// --- 9. Error Handling (Errors as Values) ---

// - Zen adopts an "errors as values" approach, avoiding traditional try/catch exceptions.
// - Functions that can fail gracefully return a sum type, typically `Result<ValueType, ErrorType>`
//   or `Option<ValueType>` (if the error is simply "value not present").

// **`Result<T, E>` Enum (Conceptual - likely from `core` or `std/result`):**
//   `Result<T, E>: enum {`
//   `    Ok(T),  // Represents success with a value of type T`
//   `    Err(E), // Represents failure with an error value of type E`
//   `}`
//   - `T` is the type of the value on success.
//   - `E` is a type describing the error (could be an enum of error codes, a struct with details, etc.).

// **Functions Returning `Result` or `Option`:**
// Example: Parsing a string to an integer
// str_utils := build.import("string") // Assume it has ParseError type and parseInt
// parseInt = (s: string) Result<int32, str_utils.ParseError> {
//     // ... parsing logic ...
//     if (success) {
//         return Result<int32, str_utils.ParseError>.Ok(parsed_value)
//     } else {
//         return Result<int32, str_utils.ParseError>.Err(error_details)
//     }
// }

// Example: Finding an item that might not exist
// findUser = (id: uint64, db: Database) Option<UserProfile> {
//     // ... database lookup ...
//     if (found) {
//         return Option<UserProfile>.Some(user_profile_instance)
//     } else {
//         return Option<UserProfile>.None
//     }
// }

// **Handling Errors with Conditional Expressions:**
// - The unified conditional expression `(Scrutinee) { Arms... }` is used to handle
//   `Result` and `Option` types.
// Example: Handling `parseInt` result
// input_str := "123"
// num_result := str_utils.parseInt(input_str) // Returns Result<int32, ParseError>

// parsed_value_or_default := (num_result) {
//     (Result.Ok(value)) => { // Destructure Ok variant to get 'value'
//         log_mod.print(log_mod.Level.Info, "Parsed successfully: $(value)")
//         value // The expression evaluates to 'value'
//     },
//     (Result.Err(err_details)) => { // Destructure Err variant
//         log_mod.print(log_mod.Level.Error, "Parse error: $(err_details.message)") // Assuming err_details has message
//         0 // Default value on error for this example
//     },
//     // No '_' needed if Result.Ok and Result.Err are the only variants.
// }

// **Error Propagation (Conceptual - "Try" operator or chaining):**
// - To simplify error propagation, a shorthand mechanism like Rust's `?` operator
//   or Go's `if err != nil { return err }` pattern might be introduced.
// - For example, `value := mightFail()?` could mean: if `mightFail()` returns `Err(e)`,
//   then the current function immediately returns `Err(e.into())` (after conversion).
//   If it returns `Ok(v)`, then `value` becomes `v`. This is TBD.
// - Without a special operator, propagation is done manually using conditional expressions.


// --- 10. Behaviors (Trait-like Contracts) ---

// - Behaviors define a set of method signatures, establishing a contract that
//   different structs or enums can implement. They enable polymorphism.
// - They do not contain data fields, only method signatures.
// - Syntax: `BehaviorName: behavior { methodName = (/* self_param */ params) returnType, ... }`
//   - The exact representation of `self` within the behavior definition (e.g., `SelfType`,
//     `Ptr<SelfType>`) needs to be finalized. It acts as a placeholder for the implementing type.

// Example:
Writer: behavior {
    // Writes bytes and returns number of bytes written or an error.
    // Assumes 'io.Error' type exists.
    write = (/* self */ data: Array<byte>) Result<usize, io.Error>
}

Closer: behavior {
    close = (/* self */) Result<void, io.Error>
}

// **Implementation of Behaviors:**
// - Structs (or enums) implement behaviors by providing concrete methods that match
//   the signatures defined in the behavior.
// - Proposed Syntax: Grouping behavior methods under a `BehaviorName:` section within
//   the struct/type definition.

// Example: Implementing Writer for a hypothetical File struct
// File: { /* ... fields like path, file_descriptor ... */
//     // ... other File methods ...
//
//     Writer: { // Methods for the Writer behavior
//         // 'self' here refers to the File instance (e.g., passed as File or Ptr<File>)
//         write = (self: Ptr<File>, data: Array<byte>) Result<usize, io.Error> {
//             // ... actual file writing logic using self.file_descriptor ...
//             // return Result<usize, io.Error>.Ok(bytes_written) or Err(...)
//         }
//     }
//
//     Closer: { // Methods for the Closer behavior
//         close = (self: Ptr<File>) Result<void, io.Error> {
//             // ... logic to close self.file_descriptor ...
//         }
//     }
// }

// - For enums or extending types from other modules (where direct modification isn't possible),
//   an `extend TypeName with BehaviorName { methods... }` syntax might be used.

// **Using Behaviors (Polymorphism):**
// - Functions can accept parameters of a behavior type, allowing them to operate
//   on any type that implements that behavior.
// - This can be achieved via:
//   1. **Generics with Behavior Constraints (Static Dispatch):**
//      `logData = (comptime T: type, source: T, output: Writer) void`
//      `    where T implements Reader, T implements Closer // Hypothetical constraint syntax`
//      `{ ... source.read(...); output.write(...); source.close(); ... }`
//      The compiler generates specialized code for each `T` (monomorphization).
//   2. **Dynamic Dispatch (Runtime Polymorphism):**
//      `processStream = (stream: Writer) void { ... stream.write(...); ... }`
//      Here, `stream` would be a "fat pointer" or "trait object" containing a pointer
//      to the data and a vtable (virtual method table) for the `Writer` methods.
// - The choice between static and dynamic dispatch has performance and code size implications.
//   Zen would need to provide syntax to control this or have clear default rules.


// --- 11. Compile-Time Metaprogramming (`comptime`) ---

// - `comptime` is a core keyword that enables code execution during compilation.
// - This allows for metaprogramming, conditional compilation, generation of lookup tables,
//   embedding resources, and pre-computation of values.

// - **`comptime { ... }` Blocks:**
//   - Code within a `comptime { ... }` block is evaluated by the compiler.
//   - The result of the block is its last expression, which becomes a compile-time constant.
//   `CompileTimePiSq := comptime { m := build.import("math"); m.PI * m.PI }`

// - **`param: comptime<ValueType>` (Comptime Value Parameters):**
//   - Declares that a function parameter must be a value known at compile time.
//   `generateLookupTable = (size: comptime<usize>) Array<int32> { /* ... */ }`
//   `MyTable := generateLookupTable(256) // '256' is a comptime value`

// - **`comptime TypeNameArg: type` (Comptime Type Parameters):**
//   - Declares that a function parameter must be a type known at compile time (for generics).
//   `createInstance = (comptime T: type) T { return T{}; /* default construct T */ }`
//   `my_point_instance := createInstance(Point)`

// - **Implicitly Comptime-Only Functions:**
//   - A function becomes implicitly comptime-only if all its parameters are comptime-qualified
//     and its body contains only operations valid at compile time (using comptime values,
//     other comptime functions, and comptime-safe intrinsics/operations).
//   `addComptime = (a: comptime<int32>, b: comptime<int32>) comptime<int32> { return a + b; }`
//   `Sum5 := addComptime(2, 3)`

// - **Accessing Core Intrinsics (via `core := @std.core`):**
//   - Fundamental compiler operations are exposed as functions in the `core` module.
//   `SizeOfInt := core.sizeOf(int32)`
//   `TypeNameStr := core.typeName(Point)`
//   `core.compileError("This configuration is invalid.")`

// --- Elegance in Conditional Expressions ---
//
// Zen encourages the most elegant, minimal, and readable style for conditional expressions.
//
// **Preferred (Flat) Style:**
// For simple value-based matches (booleans, enums, etc.), use the flat style:
//
// Example: Boolean
is_admin
| true  => "Welcome, admin!"
| _     => "Access denied."
//
// Example: Enum
colour
| .Red   => STOP
| .Blue  => GO
| .Green => GO
//
// Example: Number with guards
score
| score < 0                => "Invalid (negative)"
| score == 0               => "Zero"
| score > 0 && score < 50  => "Low"
| score >= 50 && score < 80 => "Medium"
| score >= 80 && score <= 100 => "High"
| _                        => "Unknown"
//
// **Nested Style (Allowed, but use sparingly):**
// For more complex, multi-level logic, you may nest matches, but this is rarely needed for simple cases.
//
// Example: Nested (not recommended for simple booleans)
_ => {
    (is_admin) {
        (true) => "Welcome, admin!"
        (_)    => "Access denied."
    }
}
//
// **Guidance:**
// - Use the flat style whenever possible for clarity and elegance.
// - Reserve nesting for cases where the result of one match is itself a match on another value, or for grouping related logic.
// - Zen's philosophy: minimal, direct, and expressive code is always preferred.