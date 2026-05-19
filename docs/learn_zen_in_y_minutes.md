# Learn Zen In Y Minutes

Zen is a systems language for explicit programs. Declarations are prefix-first,
blocks produce their final expression, pattern matching is the main branch
form, loops use explicit control handles, and ownership/effects are visible in
types.

Use this page as the quick language tour. Stable examples are forms to copy into
source today. Preview examples are intentionally Zen-shaped, but describe gated
surfaces such as allocator-backed strings, sync/async effects, raw memory,
actors, and comptime type matching.

## The Shape

```zen
{ io } = std

Point: {
    x: i32,
    y: i32,
}

Point.sum = (self: Point) i32 {
    self.x + self.y
}

main = () i32 {
    point = Point { x: 20, y: 22 }
    io.println("sum: ${point.sum()}")
    0
}
```

The name being introduced or changed comes first:

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

## Values

```zen
main = () i32 {
    answer = 42
    label: StaticString = "zen"

    count ::= 0
    count = count + 1

    answer + count
}
```

Local bindings are immutable by default. Use `::=` for mutable inferred locals.
After a mutable binding exists, plain `=` assigns a new value.

## Final Expressions

Zen does not use a `return` keyword. Function bodies, match arms, and nested
blocks produce values from their final expression.

```zen
max = (a: i32, b: i32) i32 {
    a > b ?
        | true { a }
        | false { b }
}
```

If you reach for a keyword exit value, put the value at the end of the block
instead.

## Types

```zen
main = () i32 {
    signed: i32 = 42
    wide: i64 = cast(signed, i64)
    ratio: f64 = 3.5
    flag: bool = true
    label: StaticString = "static text"

    flag ?
        | true { cast(wide, i32) }
        | false { cast(ratio, i32) }
}
```

Numeric conversions are explicit. Casts use prefix syntax:

```zen
cast(value, Type)
```

String literals are `StaticString`, not allocator-backed strings.

## StaticString

`StaticString` is baked into the program. It is static bytes plus a fixed byte
count known after compilation. Passing it around copies a pointer-and-length
view into program storage. It does not allocate, resize, free, or transfer heap
ownership.

```zen
title: StaticString = "Zen"

identity_static = (value: StaticString) StaticString {
    value
}
```

Use `StaticString` for literal text and other text that is part of the program
image.

## Dynamic String Preview

`String<A>` is preview syntax for owned runtime text. It is different from
`StaticString` because it has memory that can grow or be released, so the owner
must carry allocator state.

```zen
String<A>: {
    ptr: RawPtr<u8>,
    len: usize,
    capacity: usize,
    allocator: A,
}
```

A literal such as `"Zen"` never silently becomes `String<A>`. Runtime text
construction belongs on an allocator-aware API.

## Functions

```zen
add = (a: i32, b: i32) i32 {
    a + b
}

main = () i32 {
    add(20, 22)
}
```

Functions have typed parameters and an explicit result type. Use `void` for
functions that only perform effects.

## Calls And UFC

```zen
Point: {
    x: i32,
    y: i32,
}

Point.sum = (self: Point) i32 {
    self.x + self.y
}

main = () i32 {
    point = Point { x: 20, y: 22 }

    dot = point.sum()
    ufc = sum(point)

    dot + ufc
}
```

`value.method(args)` and `method(value, args)` are call-site spellings for the
same attached function. They are not alternate declaration forms.

## Structs

```zen
Person: {
    name: StaticString,
    age: i32,
}

birthday = (p: Person) Person {
    Person {
        name: p.name,
        age: p.age + 1,
    }
}
```

Struct literals name fields explicitly. Field access uses dot syntax.

## Enums And Matching

```zen
Direction:
    North,
    South,
    East,
    West

opposite = (d: Direction) Direction {
    d ?
        | North { Direction.South }
        | South { Direction.North }
        | East { Direction.West }
        | West { Direction.East }
}
```

The `?` operator is the pattern-match form for bools, enums, `Option`, and
`Result`.

```zen
Option<T>:
    None,
    Some(T)

unwrap_or<T> = (value: Option<T>, fallback: T) T {
    value ?
        | Some(inner) { inner }
        | None { fallback }
}
```

## Result And Error Handling

Zen models failure with data. There are no exceptions and no null.

```zen
Result<T, E>:
    Ok(T),
    Err(E)

divide = (a: f64, b: f64) Result<f64, StaticString> {
    b == 0.0 ?
        | true { Result<f64, StaticString>.Err("division by zero") }
        | false { Result<f64, StaticString>.Ok(a / b) }
}
```

Result and option values use the same `?` match form as booleans and enums.

## Generics

```zen
Box<T>: {
    value: T,
}

Box.get<T> = (self: Box<T>) T {
    self.value
}

identity<T> = (value: T) T {
    value
}
```

Nested generic types are written directly, such as
`Result<Option<i32>, StaticString>`.

## Behaviors

Behaviors describe required methods. Generic functions can use behavior bounds
when they need a capability.

```zen
Display: behavior {
    display: (Self) StaticString
}

Point.implements(Display) {
    display = (self: Point) StaticString {
        "Point"
    }
}

show<T: Display> = (value: T) StaticString {
    value.display()
}
```

Relationship declarations keep the changed type or behavior on the left:

```zen
Point.implements(Display)
Point.requires(Display)
PrettyDisplay.extends(Display)
```

There is no `impl Type for Behavior` spelling and no separate `extends`
keyword block.

## Loops

Zen has one loop entry form:

```zen
loop((label) {
    condition ?
        | true { label.done() }
        | false { label.next() }
})
```

The loop handle is compiler-owned. `done` and `next` are closed loop-control
verbs for that handle, not arbitrary user methods and not stringly names.

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

There is no `while`, `for`, `break`, `continue`, suffix loop, or hidden loop
result channel. Accumulated values live in explicit mutable bindings outside
the loop and are read after `done`.

## Defer

```zen
{ io } = std

main = () i32 {
    @this.defer(io.println("leaving main"))
    io.println("inside main")
    0
}
```

`defer` runs cleanup expressions before leaving the current scope.

## Imports And Modules

```zen
{ clamp, factorial } = math_utils

main = () i32 {
    clamp(factorial(4), 0, 100)
}
```

Imports use destructuring-style binding from a module path. Local files import
by module name, and dotted paths resolve through subdirectories.

## Memory And Ownership

Stable Zen does not hide heap allocation behind literals, interpolation,
method calls, or generic containers. If a value needs heap memory, the API must
show the owner and allocator path.

```zen
Label: {
    text: StaticString,
}

from_text = (text: StaticString) Label {
    Label { text: text }
}
```

Preview owner shape:

```zen
OwnedBytes<T, A>: {
    ptr: RawPtr<T>,
    len: usize,
    capacity: usize,
    allocator: A,
}
```

Pointer, length, capacity, and allocator travel together because a pointer
alone is just an address.

## Sync, Async, And Allocator Preview

`Sync` and `Async` are effect modes in type surfaces. They are not source
keywords and there is no `async fn` spelling.

Sync work returns checked data now:

```zen
read_now = (source: Source, allocator: Allocator<u8, Sync>) Result<Bytes<u8>, IoError> {
    source.read_all(allocator)
}
```

Async work returns task-shaped data:

```zen
read_later = (source: Source, allocator: Allocator<u8, Async>) Task<Result<Bytes<u8>, IoError>> {
    source.read_all_async(allocator)
}
```

Allocators follow the same outer-type rule:

```zen
Allocator<T, Sync>: behavior {
    alloc: (Self, count: usize) Result<RawPtr<T>, AllocError>
}

Allocator<T, Async>: behavior {
    alloc: (Self, count: usize) Task<Result<RawPtr<T>, AllocError>>
}
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
is complete now; `Task<Result<...>>` completes later at an explicit scheduler
boundary.

Allocator-backed construction carries the allocator through the resulting
owner:

```zen
Buffer<T, A: Allocator<T, Sync>>: {
    ptr: RawPtr<T>,
    len: usize,
    capacity: usize,
    allocator: A,
}

make_buffer<T, A: Allocator<T, Sync>> = (allocator: A, len: usize) Result<Buffer<T, A>, AllocError> {
    allocator.alloc(len) ?
        | Ok(ptr) {
            Result<Buffer<T, A>, AllocError>.Ok(Buffer<T, A> {
                ptr: ptr,
                len: len,
                capacity: len,
                allocator: allocator,
            })
        }
        | Err(error) {
            Result<Buffer<T, A>, AllocError>.Err(error)
        }
}
```

Raw allocation intrinsics such as `@builtin.raw_allocate`,
`@builtin.raw_deallocate`, and `@builtin.raw_reallocate` are compiler-owned
preview names. Stable source should not call them directly.

## Pointer, Slice, Array, Actor, And Comptime Preview

```zen
PointerViews: {
    raw: RawPtr<i32>,
    pointer: Ptr<i32>,
    mutable_pointer: MutPtr<i32>,
    slice: Slice<i32>,
    fixed: [i32; 4],
}
```

`RawPtr<T>` is the explicit raw-memory spelling used in allocator previews.
`Ptr<T>`, `MutPtr<T>`, `Slice<T>`, and `[T; N]` name pointer, mutable pointer,
slice, and fixed-array shapes.

raw pointer offset, casts, integer conversion, load, store, atomics, raw
syscalls, comptime type matching, actor framework types, and scheduler
operations are gated design work until layout, ownership, effects, and runtime
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

Result<T, E>:
    Ok(T),
    Err(E)

Display: behavior {
    display: (Self) StaticString
}

Point: {
    x: i32,
    y: i32,
}

Point.sum = (self: Point) i32 {
    self.x + self.y
}

Point.implements(Display) {
    display = (self: Point) StaticString {
        "Point"
    }
}

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

show<T: Display> = (value: T) StaticString {
    value.display()
}

main = () i32 {
    point = Point { x: 20, y: 22 }
    name: StaticString = show(point)

    io.println("${name}: ${point.sum()}")
    sum_to(10)
}
```

That example shows the core: prefix declarations, typed data, attached methods,
behavior implementations, bounded generics, expression-oriented control flow,
static text, and explicit loop control.

## More To Read

- `README.md` for the language pitch.
- `examples/README.md` for canonical runnable examples.
- `docs/V1_SPEC.md` for the full v1 contract and gated design inventory.
- `docs/PHASE_PLAN.md` for implementation sequencing.
- `docs/COMPLETION_AUDIT.md` for audit details.
