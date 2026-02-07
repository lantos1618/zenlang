# Zen: A Systems Language With Zero Keywords

*What happens when you throw out every keyword and rebuild control flow from scratch?*

---

Zen is a compiled systems programming language that makes a radical bet: **you don't need keywords**. No `if`. No `else`. No `while`, `for`, `match`, `class`, `async`, `await`, `null`, or `return`. Instead, Zen replaces all of them with a small set of composable primitives — and the result is a language that's more consistent, more expressive, and surprisingly more readable than you'd expect.

Here's a complete Zen program:

```zen
{ io } = @std

Color: Red, Green, Blue

describe = (c: Color) StaticString {
    c ?
        | .Red { "fiery red" }
        | .Green { "forest green" }
        | .Blue { "ocean blue" }
}

main = () i32 {
    io.println("The sky is ${describe(.Blue)}")
    0
}
```

No keywords. The `?` operator *is* your if/else/match/switch. Functions are just `name = (args) ReturnType { body }`. Enums are `Name: Variant1, Variant2`. And that `${...}` in the string? Compile-time interpolation.

This post covers the ideas behind Zen, what makes it different, and why some of these decisions might matter beyond this one language.

---

## The Question Mark: One Operator to Rule Them All

Most languages accumulate control flow keywords over time. Python has `if/elif/else/match/case`. Rust has `if/else/match/while/for/loop`. Each keyword is its own little grammar.

Zen has one construct: the `?` operator. It takes a value on the left and pattern arms on the right.

**Boolean branching:**

```zen
x > 0 ?
    | true { "positive" }
    | false { "non-positive" }
```

**Enum matching:**

```zen
shape ?
    | .Circle(radius) { 3.14159 * radius * radius }
    | .Rectangle(r) { r.width * r.height }
    | .Triangle(t) { 0.5 * t.base * t.height }
```

**Error handling:**

```zen
result = divide(10.0, 3.0)
result ?
    | .Ok(val) { io.println("Result: ${val}") }
    | .Err(msg) { io.println("Error: ${msg}") }
```

This isn't just syntactic sugar over a match statement. It's a design philosophy: *every decision is pattern matching*. When `if` and `match` are the same construct, you stop thinking about which one to use and start thinking about what you're matching on. It also means every conditional is exhaustive by default — the compiler checks your arms.

---

## Six Ways to Bind a Variable

Variable declaration is where Zen gets opinionated about something most languages hand-wave: mutability.

```zen
x = 42              // Immutable, type inferred
y: f64 = 3.14       // Immutable, type explicit
counter ::= 0       // Mutable, type inferred
state:: i32 = 1     // Mutable, type explicit
```

The `::` means "mutable." It's visible at the declaration site — you can scan a function and immediately see which bindings can change. No `let mut` keyword, no `var` vs `val` distinction. Just `=` for "this never changes" and `::=` for "this will."

This small choice has a large consequence: reading Zen code, you always know the mutability story at a glance.

---

## Functions Are Values, Methods Are Fiction

Zen functions use a deliberately simple syntax:

```zen
add = (a: i32, b: i32) i32 {
    a + b
}
```

There's no `fn` keyword. A function is a name bound to a callable value. This consistency extends to how methods work — or rather, how they don't.

Zen has **Universal Function Call (UFC)**. Any function whose first parameter matches a type can be called as a method on that type:

```zen
distance = (p: Point) f64 {
    (p.x * p.x + p.y * p.y)
}

// Both of these work:
distance(my_point)
my_point.distance()

// And you can chain:
5.double().add_ten().square()
```

There's no `impl` block, no `self` magic, no distinction between "functions" and "methods." If a function takes a `Point` as its first argument, you can call it on a `Point`. Period. This means every function is automatically composable — you get method chaining for free on any type, without the type's author having to opt in.

---

## No Null, No Exceptions, No Ambiguity

Zen has no `null`, no `nil`, no `undefined`. If a value might not exist, you use `Option<T>`:

```zen
find_user = (id: i32) Option<User> {
    id == 0 ?
        | true { .None }
        | false { .Some(lookup(id)) }
}
```

If an operation can fail, you use `Result<T, E>`:

```zen
divide = (a: f64, b: f64) Result<f64, StaticString> {
    b == 0.0 ?
        | true { .Err("division by zero") }
        | false { .Ok(a / b) }
}
```

Error propagation uses `.raise()` instead of a sigil:

```zen
calculate_ratio = (a: f64, b: f64, c: f64) Result<f64, StaticString> {
    first = divide(a, b).raise()
    second = divide(first, c).raise()
    .Ok(second)
}
```

If `divide` returns an `Err`, `.raise()` propagates it immediately — the function returns that `Err`. If it returns `Ok`, `.raise()` unwraps the value. This is like Rust's `?` operator, but it reads as English: "divide a by b, raise if error."

---

## Allocators Solve Function Coloring

Here's maybe the most interesting design decision in Zen. Instead of `async`/`await` keywords that infect every function signature, Zen makes the allocator determine the execution mode:

```zen
process = (alloc: Allocator) void {
    buffer = alloc.allocate(1024)
    // ... use buffer ...
    alloc.deallocate(buffer, 1024)
}

main = () i32 {
    // Same function, different execution models:
    process(Heap.sync())       // Blocking I/O
    process(Arena.async())     // Non-blocking via io_uring
    0
}
```

The same `process` function works with both synchronous and asynchronous allocation. No async keyword. No function coloring. No "you need an async version of every API." The allocator carries the execution context.

This is inspired by Zig's approach to allocators, taken further. Every heap-allocated collection takes an explicit allocator:

```zen
allocator = GPA.new()
vec = Vec<i32>.new(allocator)
vec.mut_ref().push(42)
```

It's more verbose than garbage collection, but you always know where memory comes from and how it behaves.

---

## Comptime: Metaprogramming Without Macros

Zen has a compile-time interpreter that can execute code during compilation. No macro system, no proc macros, no template metaprogramming — just regular Zen code running at compile time.

The `@std.meta` module gives comptime code access to the AST:

```zen
@comptime {
    ast = meta.parse("add = (a: i32, b: i32) i32 { a + b }")
    info = meta.type_info(ast)
    // Walk the AST, inspect fields, generate code
}
```

The meta system provides:

- `meta.parse(source)` — parse Zen source into AST nodes
- `meta.variant_name(node)` — get the AST node type ("Function", "BinaryOp", etc.)
- `meta.fields(node)` — structured field information for any AST node
- `meta.children(node)` — all child nodes for tree traversal

This means code generation is just "write Zen code that produces Zen AST nodes." No separate macro language, no hygiene rules to learn, no compile-time type system distinct from the runtime type system.

---

## Syscall-First Standard Library

The Zen stdlib doesn't wrap libc. It calls Linux syscalls directly:

```zen
{ SYS_READ, SYS_WRITE, SYS_OPEN } = @std.sys.syscall

sys_write = (fd: i32, buf_ptr: i64, count: usize) i64 {
    compiler.syscall3(SYS_WRITE, fd, buf_ptr, count)
}
```

This means the entire standard library — file I/O, networking, threading, signal handling — is implemented in Zen itself, using only compiler intrinsics. The stdlib spans 73 `.zen` files covering:

- **Core types**: Option, Result, safe pointers (Ptr, MutPtr, RawPtr)
- **Collections**: Vec, HashMap, LinkedList, Queue, Stack, Set
- **Memory**: Arena allocator, GPA, mmap-based allocation
- **I/O**: Files, directories, sockets (TCP/UDP/Unix), epoll, io_uring
- **Concurrency**: Futex-based mutex, RwLock, channels, atomics, thread spawning
- **Async**: Task scheduler, stackful coroutines, actor model
- **System**: Process management, seccomp filters, random number generation

All of it readable. All of it modifiable. If you want to understand how a mutex works, you read `stdlib/concurrency/mutex.zen` — it's a futex-based implementation in ~115 lines of Zen.

---

## The Compiler: From Source to Machine Code

The compiler is ~53,000 lines of Rust, using LLVM 18 via Inkwell for code generation. The pipeline:

```
Source → Lexer → Parser → Module Resolution → Comptime Evaluation
      → Type Checking → Monomorphization → LLVM Codegen → Machine Code
```

A few architectural highlights:

**TypeContext as a bridge.** Instead of having codegen re-infer types, the typechecker produces a `TypeContext` — a flat data structure mapping names to types — that codegen consumes. One source of truth for type information, no duplicate inference.

**Monomorphization for generics.** Like Rust, Zen uses monomorphization — `Vec<i32>` and `Vec<String>` become separate concrete types at compile time. No boxing, no vtable overhead for generic code. Bounded at 10,000 instantiations to prevent combinatorial explosion.

**JIT for development.** `zen file.zen` compiles to LLVM IR and runs it through MCJIT — no separate compile step during development. `zen file.zen -o binary` produces a native executable via object file generation and linking.

**The REPL** lets you evaluate Zen expressions interactively, compiling each input to a fresh LLVM module and executing via JIT.

---

## Built for AI Coding Assistants

This is where Zen does something no other language has done at the language level. Instead of relying solely on LSP (a stateful bidirectional protocol designed for editors), Zen ships CLI commands that output structured JSON — specifically designed for AI coding assistants:

```bash
# Full semantic analysis — every type the compiler knows
$ zen analyze app.zen --json
{
  "functions": {"main": {"params": [], "return_type": "i32"}},
  "structs": {"Point": {"fields": [{"name": "x", "type": "f64"}]}},
  "variables": {"main::count": "i32", "main::name": "StaticString"}
}

# Structured diagnostics with error codes
$ zen check app.zen --json
{
  "diagnostics": [{
    "code": "type-mismatch",
    "message": "Expected i32, found String",
    "line": 15, "column": 10
  }]
}

# Point query: what is this symbol?
$ zen query type app.zen:15:5
{"symbol": "count", "type": "i32", "kind": "variable", "scope": "main"}

# Fast symbol listing (parse-only, no typecheck)
$ zen symbols app.zen --json
{
  "symbols": [
    {"name": "Point", "kind": "struct", "line": 1},
    {"name": "main", "kind": "function", "line": 10, "signature": "() i32"}
  ]
}
```

Each command is single-shot: read file, analyze, print JSON, exit. No protocol negotiation, no persistent state. An AI agent can run `zen check --json` to get structured errors, then `zen query type` to understand specific symbols, then make a fix — all with machine-parseable output backed by the real typechecker.

The error-tolerant mode is key: even when a file has type errors, `zen analyze` and `zen query type` return partial results — everything the compiler figured out before the error. An AI agent working with broken code still gets useful type information.

---

## 2,366 Commits, 8 Months, One Vision

Zen has been in active development for 8 months. The codebase has grown to ~53K lines of Rust compiler code, ~13K lines of Zen standard library, a full LSP implementation with hover/completion/diagnostics/rename/inlay hints, and a suite of AI-native CLI tools.

Some things that aren't done yet: cross-platform support (currently Linux x86-64 only), a package manager, and self-hosting (the compiler is written in Rust, not Zen). The type system handles generics through monomorphization but doesn't yet have full variance support.

But the core thesis is already testable: a systems language without keywords, with explicit everything (allocators, mutability, error handling), and with first-class AI tooling. Whether that thesis holds up is a question only real-world usage can answer.

---

## Try It

```bash
git clone https://github.com/anoraktrend/zenlang.git
cd zenlang
cargo build --release

# Run a program
./target/release/zen examples/hello_world.zen

# Analyze a file
./target/release/zen analyze examples/error_handling.zen --json

# Start the REPL
./target/release/zen
```

The entire standard library is readable Zen code. Start with `stdlib/io/io.zen` to see how print works, or `stdlib/concurrency/mutex.zen` to see a futex implementation in 115 lines.

---

*Zen is MIT licensed and built through human-AI collaboration.*
