# Learn Zen In Y Minutes

Zen is a systems language for explicit programs: declarations are prefix-first,
blocks produce their final expression, pattern matching is the branch form,
loops use explicit control handles, and ownership/effects are visible in types.

Use this page as the quick language tour. Stable examples can be copied today.
Preview examples cover gated surfaces: allocator-backed strings, sync/async
effects, raw memory, actors, and comptime type matching.

## The Shape
Declarations are prefix-first. The name being introduced or changed comes
first:

| Need | Spell it like this |
| --- | --- |
| Import | `{ io } = std` |
| Immutable local | `name = value` |
| Mutable inferred local | `name ::= value` |
| Reassignment | `name = new_value` |
| Typed local | `name: Type = value` |
| Function | `name = (arg: Type) ResultType { final_expression }` |
| Method | `Type.method = (self: Type) ResultType { final_expression }` |
| Struct | `Name: { field: Type }` |
| Enum | `Name: Variant, Variant(Payload)` |
| Match | `value ? | Pattern { expression }` |
| Loop | `loop((l) { ... l.next() ... l.done() ... })` |
| Behavior | `Name: behavior { method: (Self) Type }` |
| Implementation | `Type.implements(Behavior) { ... }` |
| Requirement | `Type.requires(Behavior)` |
| Inheritance | `ChildBehavior.extends(ParentBehavior)` |

## Values And Results
```zen
main = () i32 {
    answer = 42
    label: StaticString = "zen"
    count ::= 0
    count = count + 1
    answer + count
}
```

Local bindings are immutable by default. Use `::=` for mutable inferred locals;
after that, plain `=` assigns a new value.

Zen does not use a `return` keyword. Function bodies, match arms, and nested
blocks produce their final expression.

```zen
max = (a: i32, b: i32) i32 {
    a > b ?
        | true { a }
        | false { b }
}
```

Numeric conversions are explicit and prefix-first: `cast(value, Type)`.
String literals are `StaticString`, not allocator-backed strings.

## StaticString
`StaticString` is baked into the program: static bytes plus a fixed byte count
known after compilation. Passing it around copies a pointer-and-length view into
program storage. It does not allocate, resize, free, or transfer heap ownership.

## Dynamic String Preview
`String<A>` is preview syntax for owned runtime text. It can grow or be
released, so the owner must carry allocator state. A literal such as `"Zen"`
never silently becomes `String<A>`; runtime text construction belongs on an
allocator-aware API.

## Calls, Structs, And Data
`value.method(args)` and `method(value, args)` are call-site spellings for the
same attached function, not alternate declaration forms. Struct literals name
fields explicitly; field access uses dot syntax.

## Enums And Matching
```zen
Direction: North, South, East, West
Option<T>: None, Some(T)
Result<T, E>: Ok(T), Err(E)

unwrap_or<T> = (value: Option<T>, fallback: T) T {
    value ?
        | Some(inner) { inner }
        | None { fallback }
}
```

The `?` operator is the pattern-match form for bools, enums, `Option`, and
`Result`.

## Result And Error Handling
Zen models failure with data. There are no exceptions and no null.

```zen
divide = (a: f64, b: f64) Result<f64, StaticString> {
    b == 0.0 ?
        | true { Result<f64, StaticString>.Err("division by zero") }
        | false { Result<f64, StaticString>.Ok(a / b) }
}
```

Nested generic types are written directly, such as
`Result<Option<i32>, StaticString>`.

## Behaviors
Behaviors describe required methods. Generic functions use behavior bounds when
they need a capability.

```zen
Display: behavior { display: (Self) StaticString }

Point.implements(Display) {
    display = (self: Point) StaticString { "Point" }
}

show<T: Display> = (value: T) StaticString {
    value.display()
}
```

Relationship declarations keep the changed type or behavior on the left:
`Point.implements(Display)`, `Point.requires(Display)`, and
`PrettyDisplay.extends(Display)`. There is no `impl Type for Behavior` spelling.

## Loops
Zen has one loop entry form. The handle is compiler-owned, and `done`/`next`
are closed control verbs for that handle, not user methods or stringly names.

Counted loop:

```zen
sum_to = (limit: i32) i32 {
    total ::= 0
    i ::= 0

    loop((l) {
        i > limit ?
            | true { l.done() }
            | false {
                total = total + i
                i = i + 1
                l.next()
            }
    })

    total
}
```

Nested loop exit:

```zen
loop((outer) {
    loop((inner) {
        stop ?
            | true { outer.done() }
            | false { inner.next() }
    })

    outer.next()
})
```

UFC loop control:

```zen
loop((l) {
    done(l)
    next(l)
})
```

There is no `while`, `for`, `break`, `continue`, suffix loop, or hidden result
channel. Accumulated values live in explicit mutable bindings outside the loop.

## Defer
`defer` runs cleanup expressions before leaving the current scope.

## Imports And Modules
Imports use destructuring-style binding from a module path; local files import
by module name, and dotted paths resolve through subdirectories.

## Memory And Ownership
Stable Zen does not hide heap allocation behind literals, interpolation, method
calls, or generic containers. Heap APIs show the owner and allocator path.

```zen
OwnedBytes<T, A>: { ptr: RawPtr<T>, len: usize, capacity: usize, allocator: A }
```

Pointer, length, capacity, and allocator travel together; a pointer alone is
just an address.

## Sync, Async, And Allocator Preview
`Sync` and `Async` are effect modes in type surfaces, not source keywords; there
is no `async fn` spelling. Sync work returns checked data now. Async work
returns task-shaped data.

```zen
read_now = (source: Source, allocator: Allocator<u8, Sync>) Result<Bytes<u8>, IoError> {
    source.read_all(allocator)
}

read_later = (source: Source, allocator: Allocator<u8, Async>) Task<Result<Bytes<u8>, IoError>> {
    source.read_all_async(allocator)
}

Allocator<T, Sync>: behavior { alloc: (Self, count: usize) Result<RawPtr<T>, AllocError> }
Allocator<T, Async>: behavior { alloc: (Self, count: usize) Task<Result<RawPtr<T>, AllocError>> }
```

Read the outer type first:

| Surface | Meaning |
| --- | --- |
| `Result<T, E>` | checked data is available now |
| `Task<Result<T, E>>` | checked data belongs to scheduled work |
| `Allocator<T, Sync>` | allocation returns now |
| `Allocator<T, Async>` | allocation returns as task-shaped work |
| `String<A>` | owned dynamic bytes plus allocator ownership |

There is no hidden conversion between sync and async allocation. `Result<...>`
is complete now; `Task<Result<...>>` completes later at a scheduler boundary.
Allocator-backed construction carries the allocator through the owner:

```zen
Buffer<T, A: Allocator<T, Sync>>: { ptr: RawPtr<T>, len: usize, capacity: usize, allocator: A }

make_buffer<T, A: Allocator<T, Sync>> = (allocator: A, len: usize) Result<Buffer<T, A>, AllocError> {
    allocator.alloc(len) ?
        | Ok(ptr) {
            buffer = Buffer<T, A> { ptr: ptr, len: len, capacity: len, allocator: allocator }
            Result<Buffer<T, A>, AllocError>.Ok(buffer)
        }
        | Err(error) { Result<Buffer<T, A>, AllocError>.Err(error) }
}
```

Raw allocation intrinsics such as `@builtin.raw_allocate`,
`@builtin.raw_deallocate`, and `@builtin.raw_reallocate` are compiler-owned
preview names; stable source should not call them directly.

## Pointer, Slice, Array, Actor, And Comptime Preview
`RawPtr<T>` is the raw-memory spelling used in allocator previews. `Ptr<T>`,
`MutPtr<T>`, `Slice<T>`, and `[T; N]` name pointer, mutable pointer, slice, and
fixed-array shapes. Raw pointer offset, casts, integer conversion, load, store,
atomics, raw syscalls, comptime type matching, actor framework types, and
scheduler operations stay gated until layout, ownership, effects, and runtime
contracts are promoted.

## Translation Cheat Sheet
| If you reach for | Use |
| --- | --- |
| keyword exit value | final expression |
| `while condition { ... }` | `loop((l) { condition ? | true { ... l.next() } | false { l.done() } })` |
| `for item in items { ... }` | explicit state plus `loop((l) { ... })` |
| `break` | `l.done()` or `done(l)` |
| `continue` | `l.next()` or `next(l)` |
| `impl Type for Behavior` | `Type.implements(Behavior) { ... }` |
| `extends Parent` keyword block | `Child.extends(Parent)` |
| `requires Behavior` keyword block | `Type.requires(Behavior)` |
| `async fn` | a function returning `Task<Result<T, E>>` or another task-shaped type |
| string literal text | `StaticString` |
| growable owned text | `String<A>` or another owner that carries allocator ownership |

## One Page Example
```zen
{ io } = std

Display: behavior { display: (Self) StaticString }

Point: { x: i32, y: i32 }

Point.sum = (self: Point) i32 {
    self.x + self.y
}

Point.implements(Display) {
    display = (self: Point) StaticString { "Point" }
}

show<T: Display> = (value: T) StaticString {
    value.display()
}

main = () i32 {
    point = Point { x: 20, y: 22 }
    io.println("${show(point)}: ${point.sum()}")
    point.sum()
}
```

That example shows prefix declarations, typed data, attached methods, behavior
implementations, bounded generics, expression-oriented control flow, static text,
and explicit loop control.

More to read: `README.md`, `examples/README.md`, and `docs/V1_SPEC.md`.
