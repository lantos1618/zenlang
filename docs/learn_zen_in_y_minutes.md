# Learn Zen In Y Minutes

Zen is a systems language built around compact declarations, explicit data
shapes, pattern matching, generics, behaviors, and predictable native output.

This guide separates the tested language surface from gated design previews.
Runnable examples live in `tests/zen/` and `examples/`.

## Hello

```zen
{ io } = std

main = () i32 {
    io.println("hello")
    0
}
```

Top-level declarations use prefix-first forms:

- imports: `{ io } = std`
- functions: `name = (...) ReturnType { ... }`
- structs: `Name: { ... }`
- enums: `Name: Variant, Variant(Payload)`
- methods: `Type.method = (...) ReturnType { ... }`
- impl blocks: `Type.impl = { ... }`
- behaviors: `Name: behavior { ... }`

## Values

```zen
main = () i32 {
    answer = 42
    message = "zen"
    ok = true

    count ::= 0
    count = count + 1

    answer + count
}
```

Local bindings are immutable by default. Use `::=` for a mutable inferred
binding and `name:: Type = value` for a mutable typed binding.

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

Numeric conversions are explicit. Mixed numeric widths need casts instead of
implicit widening.

## Functions

```zen
add = (a: i32, b: i32) i32 {
    a + b
}

main = () i32 {
    add(20, 22)
}
```

Functions have typed parameters and an explicit result type. A non-void
function's final expression must produce that result on every non-error path.

## Structs

```zen
{ io } = std

Person: {
    name: StaticString,
    age: i32,
}

birthday = (p: Person) Person {
    Person {
        name: p.name,
        age: p.age + 1
    }
}

main = () i32 {
    p = Person { name: "Ada", age: 36 }
    older = birthday(p)

    io.println("${older.name}")
    io.println("${older.age}")
    0
}
```

Struct literals name fields explicitly. Field access uses dot syntax.

## Enums And Pattern Matching

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

The `?` operator is the main pattern-match form. Enum and bool matches are
checked for missing and duplicate arms.

## Payload Enums

```zen
Option<T>:
    None,
    Some(T)

unwrap_or<T> = (value: Option<T>, fallback: T) T {
    value ?
        | Some(inner) { inner }
        | None { fallback }
}

main = () i32 {
    some = Option<i32>.Some(42)
    none = Option<i32>.None

    unwrap_or(some, unwrap_or(none, 0))
}
```

Generic enum variants are constructed with the specialized enum name.

## Result And Nested Generics

```zen
Result<T, E>:
    Ok(T),
    Err(E)

Option<T>:
    None,
    Some(T)

unwrap_result<T, E> = (value: Result<T, E>, fallback: T) T {
    value ?
        | Ok(inner) { inner }
        | Err(_) { fallback }
}

main = () i32 {
    value = Result<Option<i32>, StaticString>.Ok(Option<i32>.Some(7))

    value ?
        | Ok(option) { option.unwrap_or(0) }
        | Err(_) { 0 }
}
```

Nested generic types are written directly, such as
`Result<Option<i32>, StaticString>`.

## Methods

```zen
Point: {
    x: i32,
    y: i32,
}

Point.sum = (self: Point) i32 {
    self.x + self.y
}

main = () i32 {
    p = Point { x: 10, y: 32 }
    p.sum()
}
```

Methods are declared as `Type.method = (...) ReturnType { ... }`. Calls use dot
syntax.

## Impl Blocks

```zen
Counter: {
    value: i32,
}

Counter.impl = {
    inc = (self: Counter) Counter {
        Counter { value: self.value + 1 }
    }

    get = (self: Counter) i32 {
        self.value
    }
}

main = () i32 {
    counter = Counter { value: 41 }
    counter.inc().get()
}
```

Non-behavior `Type.impl = { ... }` groups methods under a type.

## Generics

```zen
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    self.value
}

identity<T> = (value: T) T {
    value
}

main = () i32 {
    box = Box<i32> { value: 42 }
    identity(box.get())
}
```

The compiler monomorphizes reachable generic functions, structs, enums, and
methods into concrete generated C symbols.

## Behaviors

```zen
Json: behavior {
    encode: (Self) StaticString
}

Point: {
    x: i32
}

Point.implements(Json) {
    encode = (self: Point) StaticString {
        "point"
    }
}

encode<T: Json> = (value: T) StaticString {
    value.encode()
}
```

Behaviors describe required methods. Generic functions can use behavior bounds
with `T: BehaviorName`.

## Behavior Inheritance And Requires

```zen
Json: behavior {
    encode: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)

Point.requires(Json)
```

`.extends` makes a behavior inherit parent requirements. `.requires` asserts
that a type must have a behavior implementation.

## Loops

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

Loops use prefix `loop((l) { ... })` with explicit loop-control calls:
`l.done()` exits the target loop and `l.next()` continues it. The same control
operations can be called in UFC form as `done(l)` and `next(l)`. See
`examples/05_loops.zen` for the tutorial version.

Nested loops can target an outer label directly:

```zen
nested = (stop: bool) i32 {
    count ::= 0

    loop((outer) {
        loop((inner) {
            stop ?
                | true { outer.done() }
                | false {
                    count = count + 1
                    inner.next()
                }
        })

        outer.next()
    })

    count
}
```

The loop controls are ordinary calls, so UFC form is equivalent:

```zen
single_step = (ready: bool) i32 {
    value ::= 0

    loop((l) {
        ready ?
            | true { done(l) }
            | false {
                value = value + 1
                next(l)
            }
    })

    value
}
```

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
by module name, and dotted paths resolve through subdirectories. See
`examples/project/main.zen` for the project-style example.

## Static And Dynamic Strings

```zen
{ io } = std

main = () i32 {
    name: StaticString = "Zen"
    io.println("hello ${name}")
    0
}
```

`StaticString` is baked into the program. It points at static storage and keeps
its length with the value, so a literal can be passed around without allocating
or changing ownership.

The allocator-backed String type is dynamic: it owns memory, can grow
or be built at runtime, and must be created through allocator-aware APIs once
the allocator model is promoted.

String interpolation embeds expressions with `${...}` and currently produces
the tested static output shape used by the C backend fixtures.

## Gated Preview: Sync, Async, And Allocators

The following syntax and APIs are gated design goals, not stable compiler
behavior yet. They are included here because they are central to the intended
language shape.

```zen
Allocator<T, Sync>: behavior {
    alloc: (Self, count: usize) Result<RawPtr<T>, AllocError>
}

Allocator<T, Async>: behavior {
    alloc: (Self, count: usize) Task<Result<RawPtr<T>, AllocError>>
}

Buffer<T, A: Allocator<T, Sync>>: {
    ptr: RawPtr<T>,
    len: usize,
    allocator: A,
}

make_buffer<T, A: Allocator<T, Sync>> = (allocator: A, len: usize) Result<Buffer<T, A>, AllocError> {
    ptr = allocator.alloc(len).raise()
    Result<Buffer<T, A>, AllocError>.Ok(Buffer<T, A> {
        ptr: ptr,
        len: len,
        allocator: allocator
    })
}
```

The intended model is:

- `Sync` and `Async` are real effects, not marker-only names.
- Sync code cannot call async operations without an explicit runtime boundary.
- `Allocator<T, Sync>` and `Allocator<T, Async>` are distinct capabilities.
- Allocation returns explicit `Result` or task-shaped results, not hidden
  exceptions.
- `.raise()` is the planned Result propagation operator, but it is gated until
  typechecked propagation and lowering are implemented.

For the current contract and gate status, see `docs/V1_SPEC.md`.

## More To Read

- `examples/01_hello_world.zen`
- `examples/02_variables_and_types.zen`
- `examples/03_pattern_matching.zen`
- `examples/04_structs_and_methods.zen`
- `examples/05_loops.zen`
- `examples/06_error_handling.zen`
- `examples/project/main.zen`
- `tests/zen/` for executable compiler fixtures
- `docs/V1_SPEC.md` for implemented versus gated language promises
