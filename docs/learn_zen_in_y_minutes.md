# Learn Zen In Y Minutes

Zen is a systems language built around prefix-first declarations, explicit data
shapes, pattern matching, generics, behaviors, visible ownership, and
predictable native output.

Runnable examples live in `tests/zen/` and `examples/`. Gated design previews
are called out explicitly.

Read this as two layers:

- stable source forms you can use in examples today;
- gated design previews that show intended syntax, but currently compile to
  feature-gate diagnostics instead of pretending the feature exists.

The stable tour avoids `return`, `break`, `continue`, hidden allocation,
exceptions, and null. Values come from final expressions, loop control is an
explicit call, and heap ownership has to appear in the type/API surface.

Quick map:

- declarations are prefix-first: the name appears before the operation;
- calls are ordinary prefix calls, with dot and UFC spellings where a receiver
  makes the code clearer;
- values come from final expressions, not `return`;
- branching uses `?` pattern matching for bools, enums, `Option`, and `Result`;
- loops use one prefix form, `loop((label) { ... })`, with explicit
  compiler-owned `label.done()` and `label.next()` control calls;
- type relationships are receiver-first declarations such as
  `Point.implements(Json)`, `PrettyJson.extends(Json)`, and
  `Point.requires(Json)`;
- static text is `StaticString`; allocator-backed `String` is the dynamic
  owned text shape and is still gated;
- sync, async, allocators, owned dynamic memory, and raw memory are explicit
  design surfaces, with unstable spellings called out as gated previews.

Read examples literally. If text is static, the type says `StaticString`. If a
value can allocate, grow, or escape with heap ownership, the allocator appears
in the type that owns it. If control flow exits early, the source says so with a
data value, a final expression, or a loop-control call.

## Use This Mental Model

Zen keeps the important program edges visible:

- Control is explicit. Functions, matches, and blocks produce values from final
  expressions. Loops enter through `loop((l) { ... })` and leave through
  compiler-owned `done`/`next` calls.
- Text ownership is explicit. `StaticString` is baked program text. Dynamic
  `String<A>` is allocator-backed owned text and remains gated until allocator
  ownership is promoted.
- Effects are explicit. Sync work returns a direct value such as
  `Result<T, E>`. Async work returns a task-shaped value such as
  `Task<Result<T, E>>`.
- Behavior relationships are explicit. Use `Type.implements(Behavior)`,
  `Type.requires(Behavior)`, and `Child.extends(Parent)`.
- Tooling truth comes from the compiler. JSON views are emitted from source;
  hand-authored JSON is not accepted as checked program state.

No in-between keyword phrases are part of the stable tutorial syntax. If a
form reads like a borrowed phrase from another language, translate it into the
receiver-first or prefix-first Zen form.

## Translation Cheat Sheet

| If you reach for | Use |
| --- | --- |
| `return` plus a value | put the value in the final expression |
| `while condition { ... }` | `loop((l) { condition ? | true { ... l.next() } | false { l.done() } })` |
| `for item in items { ... }` | explicit state plus `loop((l) { ... })` |
| `break` | `l.done()` or `done(l)` |
| `continue` | `l.next()` or `next(l)` |
| `impl Type for Behavior` | `Type.implements(Behavior) { ... }` |
| `async fn` | a function returning `Task<Result<T, E>>` or another task-shaped type |
| string literal as owned text | `StaticString` |
| growable owned text | `String<A>` with allocator ownership |

## Copy These Forms First

Zen is prefix-first at declaration and control boundaries. Use these spellings
as the canonical forms before reaching for alternatives:

```zen
{ io } = std

StaticLabel: {
    text: StaticString,
}

Json: behavior {
    encode: (Self) StaticString
}

Point: {
    x: i32,
    y: i32,
}

Point.sum = (self: Point) i32 {
    self.x + self.y
}

Point.implements(Json) {
    encode = (self: Point) StaticString {
        "point"
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
```

The important boundaries:

- StaticString is not a String. It is static text baked into the program:
  a stable pointer and a constant length, with no allocator and no ownership.
- `String<A>` is dynamic text. It owns runtime memory and must carry the
  allocator capability that can release or grow that memory.
- loop control is prefix-only: enter with `loop((l) { ... })`, then call
  `l.done()`, `l.next()`, `done(l)`, or `next(l)`.
- behavior relationships stay attached to the thing being changed:
  `Point.implements(Json)`, `PrettyJson.extends(Json)`, and
  `Point.requires(Json)`.
- There is no `impl Type for Behavior` spelling. There is no source-level
  `async` keyword. There is no `async fn` spelling in the stable tour. Sync,
  async, and allocator behavior is visible through types such as
  `Allocator<T, Sync>`, `Allocator<T, Async>`, `Result<T, E>`, and
  `Task<Result<T, E>>`.

If you only remember one page, make it this one:

```zen
{ io } = std

Box<T>: {
    value: T
}

identity<T> = (value: T) T {
    value
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

main = () i32 {
    label: StaticString = "baked into the program"
    box = Box<i32> { value: identity(41) }
    io.println("${label}")
    box.value + 1
}
```

The shape is intentional:

- `StaticString` is literal/static text: pointer plus length into program
  storage, with no allocator.
- `String` is runtime-owned dynamic text: pointer, length, capacity, and an
  allocator capability. It is a gated allocator-backed shape, not what string
  literals produce.
- Static text and dynamic text are different types, not different sizes of the
  same type.
- `loop((l) { ... })` is the loop form. Exit with `l.done()` or `done(l)`;
  continue with `l.next()` or `next(l)`.
- `Sync`, `Async`, and `Allocator<T, Mode>` live in type signatures so effects
  and ownership are visible.
- `return`, `break`, `continue`, exceptions, null, and hidden allocation are
  not the stable source model.

## Control Flow At A Glance

Zen control flow is expression-oriented and prefix-first at the boundary where
control begins:

| Need | Stable form |
| --- | --- |
| Choose by bool, enum, `Option`, or `Result` | `value ? | Pattern { expr }` |
| Produce a function result | put the value in the final expression |
| Repeat work | `loop((l) { ... })` |
| Continue a loop | `l.next()` or `next(l)` |
| Exit a loop | `l.done()` or `done(l)` |
| Fail or be absent | use `Result<T, E>` or `Option<T>` |

Loop syntax is prefix-only. No in-between loop syntax exists. There is no
`while (...) { ... }`, no `for item in items { ... }`, and no body-first loop
spelling. Convert those forms to `loop((l) { ... })` with explicit state and
explicit `done`/`next` edges.

At a glance, the major surfaces look like this:

```zen
// Static text: baked into the program, no allocator.
name: StaticString = "Zen"

// Dynamic text preview: owned bytes plus the allocator that owns them.
String<A>: {
    ptr: RawPtr<u8>,
    len: usize,
    capacity: usize,
    allocator: A,
}

// Counted, sentinel, and infinite loops all use the same prefix form.
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

// Nested loop control names the target explicitly.
nested = (stop: bool) void {
    loop((outer) {
        loop((inner) {
            stop ?
                | true { outer.done() }
                | false { inner.next() }
        })

        outer.next()
    })
}

// UFC loop control keeps the verb first.
step = (done_now: bool) void {
    loop((l) {
        done_now ?
            | true { done(l) }
            | false { next(l) }
    })
}
```

The gated effect and allocator previews use ordinary type positions rather than
new control keywords:

```zen
read_now = (source: Source, allocator: Allocator<u8, Sync>) Result<Bytes<u8>, IoError> {
    source.read_all(allocator)
}

read_later = (source: Source, allocator: Allocator<u8, Async>) Task<Result<Bytes<u8>, IoError>> {
    source.read_all_async(allocator)
}

Allocator<T, Sync>: behavior {
    alloc: (Self, count: usize) Result<RawPtr<T>, AllocError>
}

Allocator<T, Async>: behavior {
    alloc: (Self, count: usize) Task<Result<RawPtr<T>, AllocError>>
}
```

That is the rule of thumb: sync returns a direct checked value, async returns a
task-shaped checked value, and allocation is visible because allocator
ownership travels with the pointer-backed value.

The core syntax to keep in your head:

```zen
{ name } = module.path

Name: { field: Type }

Name: Variant, Variant(Payload)

function = (arg: Type) ReturnType {
    final_expression
}

Type.method = (self: Type) ReturnType {
    final_expression
}

value ?
    | Pattern { expression }
    | Other { expression }

loop((label) {
    condition ?
        | true { label.done() }
        | false { label.next() }
})
```

Stable Zen is deliberately small. Most code is built from:

| Need | Spell it like this |
| --- | --- |
| Import | `{ io } = std` |
| Immutable local | `name = value` |
| Mutable inferred local | `name ::= value` |
| Reassignment | `name = new_value` |
| Typed local | `name: Type = value` |
| Function | `name = (arg: Type) ReturnType { expr }` |
| Method | `Type.method = (self: Type) ReturnType { expr }` |
| Struct | `Name: { field: Type }` |
| Enum | `Name: Variant, Variant(Payload)` |
| Match | `value ? | Pattern { expr }` |
| Loop | `loop((l) { ... l.next() ... l.done() ... })` |
| Behavior | `Name: behavior { method: (Self) Type }` |
| Implementation | `Type.implements(Behavior) { ... }` |
| Required behavior | `Type.requires(Behavior)` |
| Behavior inheritance | `ChildBehavior.extends(ParentBehavior)` |
| Generic bound | `name<T: Behavior> = (...) T { ... }` |

If a feature needs hidden allocation, implicit scheduling, exceptions, null, or
statement-level early exits, it is not part of the stable shape. Zen makes
those things visible with data types, final expressions, loop-control calls,
and allocator or effect parameters.

Zen has no suffix/body-first control form in the stable tour. Dot syntax and
UFC are receiver-call spellings; declarations and control forms stay
prefix-first.

## The Shape

Zen reads name-first. A declaration starts with the thing being introduced or
attached, then gives its shape:

```zen
{ io } = std

Point: {
    x: i32,
    y: i32,
}

Point.length = (self: Point) i32 {
    self.x + self.y
}

main = () i32 {
    point = Point { x: 20, y: 22 }
    point.length()
}
```

The common forms are:

- `{ name } = module.path` imports names from a module.
- `Name: { field: Type }` declares a struct.
- `Name: Variant, Variant(Payload)` declares an enum.
- `name = (...) ReturnType { ... }` declares a function.
- `Type.method = (...) ReturnType { ... }` declares an attached method.
- `value.method(args)` and `method(value, args)` are the same receiver call.
- `Name: behavior { ... }` declares a behavior contract.
- `Type.implements(Behavior) { ... }` gives a type that behavior.

## Hello

```zen
{ io } = std

main = () i32 {
    io.println("hello")
    0
}
```

Top-level declarations use prefix-first forms. The thing being introduced is
visible first, and the operation follows:

- imports: `{ io } = std`
- functions: `name = (...) ReturnType { ... }`
- structs: `Name: { ... }`
- enums: `Name: Variant, Variant(Payload)`
- methods: `Type.method = (...) ReturnType { ... }`
- behaviors: `Name: behavior { ... }`

Top-level `pub` exposes a declaration across modules:

```zen
pub Point: {
    x: i32,
    y: i32,
}

pub origin = () Point {
    Point { x: 0, y: 0 }
}
```

## Comments

```zen
// Line comments use two slashes.

/*
Block comments can document a whole declaration.
*/
main = () i32 {
    0
}
```

Comments are not part of the syntax tree emitted for semantic tools.

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

Local bindings are immutable by default. Use `::=` when the value is meant to
change in the current scope.

## Assignment And Mutation

```zen
main = () i32 {
    immutable = 40

    inferred ::= 1
    inferred = inferred + 1

    typed: i32 = 1

    immutable + typed
}
```

Use `::=` for a mutable inferred binding. After a mutable binding exists,
plain `=` assigns a new value. Plain `name = value` creates an immutable
binding, and `name: Type = value` creates an immutable typed binding.

The rule is local and visible: mutation starts at the binding site. A reader
does not have to search for later assignments to learn whether a value can
change.

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

String literals are `StaticString` values. The bytes are baked into the
program image, and the value carries a pointer plus length for that static
data. Think of it as compile-time-sized static text: the program knows where
the bytes live, and using the literal does not allocate.

Dynamic `String` is different. It is allocator-backed owned text: it can own
runtime memory, grow, and change contents, so constructing one requires an
allocator path instead of happening implicitly from a literal.

Use `StaticString` for fixed program text and diagnostics. Use allocator-backed
`String` only when the API really needs runtime-owned, mutable, or growable
text.

The quick distinction:

| Text shape | Owns memory | Needs allocator | Can grow | Typical use |
| --- | --- | --- | --- | --- |
| `StaticString` | No | No | No | literals, labels, diagnostics |
| `String<A>` | Yes | Yes | Yes | runtime text, builders, owned buffers |

That means `"Zen"` is a `StaticString`, not a small `String`. A string literal
has known bytes in the program image; a dynamic string has runtime storage and
must remember which allocator owns that storage.

## Operators And Casts

```zen
main = () i32 {
    a = 10
    b = 3

    sum = a + b
    diff = a - b
    product = a * b
    quotient = a / b

    same = sum == 13
    ordered = product > quotient

    same && ordered ?
        | true { cast(product, i32) }
        | false { cast(diff, i32) }
}
```

Arithmetic and comparison operators are ordinary expressions. Casts use prefix
`cast(value, Type)` syntax so type-changing operations stay visible at the call
site.

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
Use `void` for functions that only perform effects. `return` is not part of
Zen source.

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

Attached functions can be called with dot syntax or uniform function call
syntax. Dot syntax keeps the receiver first when the operation reads like a
method. UFC keeps the operation first when that is clearer. These are call-site
spellings, not alternate declaration forms. The receiver is still an explicit
argument in the declared function type.

## Blocks Produce Their Final Expression

```zen
max = (a: i32, b: i32) i32 {
    a > b ?
        | true { a }
        | false { b }
}

main = () i32 {
    max(10, 42)
}
```

Zen does not use a `return` keyword. Function bodies, match arms, and nested
blocks produce values from their final expression. Use `Result` or `Option`
when a path can fail or be absent.

```zen
classify = (value: i32) StaticString {
    value == 0 ?
        | true { "zero" }
        | false {
            value > 0 ?
                | true { "positive" }
                | false { "negative" }
        }
}
```

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

Pattern arms are blocks, so nested decisions still stay expression-oriented:

```zen
sign = (value: i32) StaticString {
    value == 0 ?
        | true { "zero" }
        | false {
            value > 0 ?
                | true { "positive" }
                | false { "negative" }
        }
}
```

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

## Error Handling

Zen models absence and failure with ordinary data. No exceptions are thrown
behind a call boundary. No null value exists to check at runtime.

```zen
Result<T, E>:
    Ok(T),
    Err(E)

Option<T>:
    None,
    Some(T)

divide = (a: f64, b: f64) Result<f64, StaticString> {
    b == 0.0 ?
        | true { Result<f64, StaticString>.Err("division by zero") }
        | false { Result<f64, StaticString>.Ok(a / b) }
}

ratio_or_zero = (a: f64, b: f64) f64 {
    divide(a, b) ?
        | Ok(value) { value }
        | Err(_) { 0.0 }
}
```

Result and option values are handled with the same `?` match form used for
booleans and enums. The final expression of each arm is the arm result.

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
syntax. A method is still a function with an explicit receiver; Zen does not
hide data behind classes or an object model.

## Attached Methods

```zen
Counter: {
    value: i32,
}

Counter.inc = (self: Counter) Counter {
    Counter { value: self.value + 1 }
}

Counter.get = (self: Counter) i32 {
    self.value
}

main = () i32 {
    counter = Counter { value: 41 }
    counter.inc().get()
}
```

Prefer direct `Type.method = ...` declarations in public examples. They keep
the receiver type on the left, avoid extra grouping syntax, and line up with
dot and UFC calls.

Generic attached methods use the same receiver-first shape:

```zen
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    self.value
}
```

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

Behavior bounds can appear on generic parameters when a generic function needs
a capability:

```zen
Display: behavior {
    display: (Self) StaticString
}

show<T: Display> = (value: T) StaticString {
    value.display()
}
```

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
with `T: BehaviorName`. The bound gives the generic body permission to call the
required methods.

Behavior association is explicit:

```zen
Point.implements(Json) {
    encode = (self: Point) StaticString {
        "point"
    }
}
```

This keeps capability lookup visible at the type boundary.

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

These are receiver-first relationship declarations, not free-floating
keywords. Zen keeps the left-hand side as the thing being changed:

| Relationship | Meaning |
| --- | --- |
| `Point.implements(Json)` | `Point` provides `Json` methods |
| `PrettyJson.extends(Json)` | `PrettyJson` includes `Json` requirements |
| `Point.requires(Json)` | `Point` is required to have `Json` available |

Read them as declarations attached to the left side:

```zen
Point.implements(Json) {
    encode = (self: Point) StaticString {
        "point"
    }
}

PrettyJson.extends(Json)
Point.requires(PrettyJson)
```

That keeps the syntax prefix/receiver-first without borrowing an `impl Type for
Behavior` or `behavior Child extends Parent` form. The relationship is the
operation, and the receiver is the type or behavior being updated.

## Loops

Zen has one loop form. There are no `for` or `while` keywords, and loop exits
are explicit calls instead of `break` or `continue`. The spelling is
prefix-only: call `loop`, pass a loop-control parameter, then choose the next
edge with `done` or `next`.

The loop parameter is a control handle, not a user-defined object. `done` and
`next` are compiler-owned loop-control verbs for that handle. They are not
arbitrary user methods on a library object. That keeps loop control visible
without adding more statement keywords.

This is the core loop shape:

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

Loops use prefix `loop((l) { ... })` with explicit loop-control calls. The
control parameter names the loop target: `l.done()` exits that target and
`l.next()` continues it. The compiler recognizes only the control verbs for
the loop handle here; this is not general user-defined method dispatch. See
`examples/05_loops.zen` for the runnable tutorial version.

`break` and `continue` are not Zen source forms. Use `l.done()` and `l.next()`
or their UFC forms instead.

Nested loops can target an outer label directly:

```zen
nested = (stop: bool) void {
    loop((outer) {
        loop((inner) {
            stop ?
                | true { outer.done() }
                | false { inner.next() }
        })

        outer.next()
    })
}
```

Loop controls also support the narrow UFC form, which keeps the operation
first while still naming the loop handle explicitly:

```zen
step = (done_now: bool) void {
    loop((l) {
        done_now ?
            | true { done(l) }
            | false { next(l) }
    })
}
```

The same target rule applies in nested loops: `done(outer)` exits the outer
loop, and `next(inner)` continues only the inner loop. UFC loop control is
limited to these compiler-owned verbs; it does not make loop handles ordinary
objects.

The complete stable loop surface is:

- `loop((l) { ... })` starts a loop and binds a control handle.
- `l.done()` exits that loop.
- `l.next()` continues that loop.
- `done(l)` and `next(l)` are the equivalent UFC forms.
- A nested loop can control an outer loop with `outer.done()` or `done(outer)`.
- There is no suffix/body-first loop spelling; `loop(...)` is the prefix entry
  point.

Loops do not have a hidden result channel. Accumulated values live in explicit
mutable bindings outside the loop and are read after `done`.

Use this recipe when converting `while condition` code:

```zen
while_like = (limit: i32) i32 {
    i ::= 0

    loop((l) {
        i < limit ?
            | true {
                i = i + 1
                l.next()
            }
            | false { l.done() }
    })

    i
}
```

Use this recipe when converting `for i in 0..limit` code:

```zen
for_like = (limit: i32) i32 {
    total ::= 0
    i ::= 0

    loop((l) {
        i >= limit ?
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

That gives Zen one answer for counted loops, sentinel loops, and nested exits:

```zen
find_first = (limit: i32, needle: i32) Option<i32> {
    i ::= 0
    found ::= Option<i32>.None

    loop((l) {
        i >= limit ?
            | true { l.done() }
            | false {
                i == needle ?
                    | true {
                        found = Option<i32>.Some(i)
                        l.done()
                    }
                    | false {
                        i = i + 1
                        l.next()
                    }
            }
    })

    found
}
```

There is no hidden loop result channel here. The value that survives the loop is
an ordinary binding, and every control edge names whether the loop is done or
continues.

Loop recipes use the same skeleton:

```zen
// Count up until a bound.
count = (limit: i32) i32 {
    i ::= 0
    loop((l) {
        i >= limit ?
            | true { l.done() }
            | false {
                i = i + 1
                l.next()
            }
    })
    i
}

// Stop on a sentinel.
scan = (limit: i32, stop_at: i32) Option<i32> {
    i ::= 0
    found ::= Option<i32>.None
    loop((l) {
        i >= limit ?
            | true { l.done() }
            | false {
                i == stop_at ?
                    | true {
                        found = Option<i32>.Some(i)
                        l.done()
                    }
                    | false {
                        i = i + 1
                        l.next()
                    }
            }
    })
    found
}
```

The shape is intentionally regular: counted loops, sentinel loops, nested
exits, and UFC control calls all use the same loop entry and the same two
compiler-owned verbs.

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

Imported declarations have to be public in the source module:

```zen
pub clamp = (value: i32, low: i32, high: i32) i32 {
    value < low ?
        | true { low }
        | false {
            value > high ?
                | true { high }
                | false { value }
        }
}
```

## Memory And Ownership

Allocation is explicit in Zen's language model. The stable subset does not hide
heap allocation behind literals, interpolation, method calls, or generic
containers.

```zen
Label: {
    text: StaticString,
}

from_text = (text: StaticString) Label {
    Label { text: text }
}
```

Values name their shape directly. Static text is a non-owning program value.
Owned dynamic storage is modeled with allocator-aware types once typed
allocators are promoted. Until then, the compiler rejects allocator-backed
source types instead of pretending allocation is free.

The practical rule is simple: if a value needs heap memory, the API must show
the allocator path. Literals and interpolation do not smuggle allocation into
ordinary expressions.

A static value can be copied around without an owner:

```zen
LogLine: {
    level: StaticString,
    message: StaticString,
}

info = (message: StaticString) LogLine {
    LogLine { level: "info", message: message }
}
```

A dynamic value must carry ownership in its type. The intended dynamic form is
allocator-shaped. This is not stable source yet; it is the shape Zen is
promoting through the gated allocator work:

```zen
OwnedBytes<T, A>: {
    ptr: RawPtr<T>,
    len: usize,
    allocator: A,
}
```

The important part is not the exact container name. The important part is that
the pointer, length, and allocator capability travel together.

Allocator-aware owners should be designed so the allocation cannot be separated
from the capability that can release or grow it:

```zen
Buffer<T, A>: {
    ptr: RawPtr<T>,
    len: usize,
    capacity: usize,
    allocator: A,
}
```

Passing only `RawPtr<T>` around loses ownership information. Passing
`Buffer<T, A>` keeps the raw address, size facts, and allocator capability in
one value.

## Pointer, Slice, And Array Types

Pointer, slice, and array type syntax exists for signatures and compiler-owned
layout work:

```zen
RawPtr<T>: {
    address: usize,
}

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
slice, and fixed-array shapes. The syntax is intentionally separate from
ownership: raw pointer offset, casts, integer conversion, load, and store
operations are gated until provenance, layout, and ownership rules are
promoted.

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
or changing ownership. Its bytes live in the program image; the value is a
stable pointer-and-length view and does not own or free memory.

The allocator-backed String type is dynamic: it owns memory, carries
allocator-managed length and capacity, can grow, can be built at runtime, and
must be created through allocator-aware APIs once the allocator model is
promoted. Until that ownership path exists, source-level `String` annotations
are gated; use `StaticString` for literal/static text.

That distinction is deliberate:

- `StaticString` is a non-owning view into program storage.
- `String` is owned dynamic memory and therefore needs allocator ownership.
- `StaticString` has stable bytes and length; `String` has allocator-managed
  capacity, length, and storage.
- A literal such as `"Zen"` does not allocate a `String`.
- APIs that need to store or mutate text should say so with an allocator-backed
  type rather than accepting a literal and allocating invisibly.

String interpolation embeds expressions with `${...}`. In stable examples it is
non-owning and does not allocate a dynamic `String`; only literal bytes are
guaranteed to be baked into program storage.

A good API makes the choice visible:

```zen
LogLine: {
    level: StaticString,
    message: StaticString,
}

String<A>: {
    ptr: RawPtr<u8>,
    len: usize,
    capacity: usize,
    allocator: A,
}
```

The first shape stores text that already lives in the program. The second
shape owns runtime bytes, so it must carry the allocator that owns those bytes.
That is the difference between static text and dynamic text in Zen.

## Gated Preview: Sync, Async, And Allocators

The following syntax and APIs are gated design goals, not stable compiler
behavior yet. They are included here because they are central to the intended
language shape: allocation is explicit, async work is effect-aware, and sync
code cannot accidentally call async operations. Current compiler paths reject
these spellings with feature-gate diagnostics instead of treating them as
ordinary unknown names.

### Sync/Async/Allocator Quick Rules

- `Sync` APIs compute now and produce direct checked data.
- `Async` APIs describe later work and produce a task-shaped value.
- `Allocator<T, Sync>` allocates now and returns `Result<RawPtr<T>, AllocError>`.
- `Allocator<T, Async>` allocates later and returns
  `Task<Result<RawPtr<T>, AllocError>>`.
- allocator-backed owners keep the allocator with the pointer, length, and
  capacity facts.
- async work returns a task-shaped value instead of hiding scheduler work inside
  an ordinary result.
- loop handles are compiler-owned; their control verbs are `done` and `next`,
  not arbitrary user methods.

### Sync And Async Preview

`Sync` and `Async` are effect modes. They are part of the type contract, not
marker-only names. The intended rule is that a function either runs in a sync
context or returns task-shaped async work explicitly:

```zen
read_now = (source: Source, allocator: Allocator<u8, Sync>) Result<Bytes<u8>, IoError> {
    source.read_all(allocator)
}

read_later = (source: Source, allocator: Allocator<u8, Async>) Task<Result<Bytes<u8>, IoError>> {
    source.read_all_async(allocator)
}
```

Sync code either stays sync or crosses an explicit runtime boundary. It does
not implicitly await async work, schedule tasks behind a call, or hide task
creation in a normal result type. Planned `.await()` and async scheduler
intrinsics are gated until task lowering and effect checking are promoted.

The intended source rule is:

- a `Sync` API returns the result it computed now;
- an `Async` API returns a `Task<...>` that represents work to run later;
- sync code can call sync code directly;
- async code needs an explicit task/runtime boundary before sync callers can
  observe its result;
- allocator and scheduler APIs should expose their effect mode in the type
  surface instead of hiding it behind a normal call.

This keeps async visible at the type boundary:

```zen
start = (source: Source, allocator: Allocator<u8, Async>) Task<Result<Bytes<u8>, IoError>> {
    source.read_all_async(allocator)
}
```

The caller sees the difference immediately:

```zen
use_sync = (source: Source, allocator: Allocator<u8, Sync>) Result<Bytes<u8>, IoError> {
    read_now(source, allocator)
}

use_async = (source: Source, allocator: Allocator<u8, Async>) Task<Result<Bytes<u8>, IoError>> {
    read_later(source, allocator)
}
```

There is no source-level `async` keyword in the stable tour. The preview keeps
the effect in ordinary Zen types: `Sync`, `Async`, `Task<T>`, and allocator
capabilities.

That means these are different APIs, not the same function with a hidden
scheduler decision:

```zen
load_config = (path: StaticString, allocator: Allocator<u8, Sync>) Result<Bytes<u8>, IoError> {
    read_file(path, allocator)
}

load_config_later =
    (path: StaticString, allocator: Allocator<u8, Async>) Task<Result<Bytes<u8>, IoError>> {
    read_file_async(path, allocator)
}
```

The sync version returns checked data. The async version returns a task that
will eventually produce checked data. A caller can see the difference without
opening the implementation.

The useful mental model is:

```zen
sync_read = (source: Source, allocator: Allocator<u8, Sync>) Result<Bytes<u8>, IoError> {
    source.read_all(allocator)
}

async_read = (source: Source, allocator: Allocator<u8, Async>) Task<Result<Bytes<u8>, IoError>> {
    source.read_all_async(allocator)
}
```

The call site should not need to guess whether work runs now, later, or on a
scheduler. The result type and allocator mode say that directly.

Read these preview signatures literally:

| Surface | Shape | Meaning |
| --- | --- | --- |
| Sync function | `(args...) Result<T, E>` | the work runs now and returns checked data |
| Async function | `(args...) Task<Result<T, E>>` | the work is task-shaped and completes later |
| Sync allocator | `Allocator<T, Sync>` | allocation returns `Result<RawPtr<T>, AllocError>` now |
| Async allocator | `Allocator<T, Async>` | allocation returns `Task<Result<RawPtr<T>, AllocError>>` later |

### Allocator Preview

Allocators are typed by the value they allocate and by the effect mode they run
under. `Allocator<T, Sync>` and `Allocator<T, Async>` are distinct
capabilities, so the type signature says whether allocation is immediate or
task-shaped:

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
    allocator.alloc(len) ?
        | Ok(ptr) {
            Result<Buffer<T, A>, AllocError>.Ok(Buffer<T, A> {
                ptr: ptr,
                len: len,
                allocator: allocator
            })
        }
        | Err(error) { Result<Buffer<T, A>, AllocError>.Err(error) }
}
```

A sync allocator returns a direct checked result. An async allocator returns a
task-shaped result and has to stay in async-capable code:

```zen
make_async_buffer<T, A: Allocator<T, Async>> =
    (allocator: A, len: usize) Task<Result<Buffer<T, A>, AllocError>> {
    allocator.alloc(len)
}
```

Raw allocation intrinsics such as `@builtin.raw_allocate(...)`,
`@builtin.raw_deallocate(...)`, and `@builtin.raw_reallocate(...)` are also
gated. They exist as compiler-owned names so allocator diagnostics can be
specific, but stable source code should not call them yet. Public code should
prefer typed allocator capabilities once allocation is promoted.

The model is:

- `Sync` and `Async` are real effects, not marker-only names.
- Sync code cannot call async operations without an explicit runtime boundary.
- `Allocator<T, Sync>` and `Allocator<T, Async>` are distinct capabilities.
- Sync allocation returns `Result` directly.
- Async allocation returns `Task<Result<...>>`.
- Dynamic memory ownership is visible in the returned type.
- Allocation returns explicit `Result` or task-shaped results, not hidden
  exceptions.
- `.raise()` is the planned Result propagation operator, but it is gated until
  typechecked propagation and lowering are implemented.
- Task chaining and async scheduler APIs are gated until Sync/Async effect
  checking and task lowering are implemented.

The allocator is part of the owner. A buffer or dynamic string is not just a
pointer and a length; it must also carry the capability that can release or
grow that storage.

The owner should be the value that keeps those facts together:

```zen
Bytes<T, A>: {
    ptr: RawPtr<T>,
    len: usize,
    allocator: A,
}

from_raw<T, A: Allocator<T, Sync>> =
    (ptr: RawPtr<T>, len: usize, allocator: A) Bytes<T, A> {
    Bytes<T, A> {
        ptr: ptr,
        len: len,
        allocator: allocator
    }
}
```

Passing a raw pointer alone is just an address. Passing `Bytes<T, A>` preserves
the address, length, and allocator capability together, which is the minimum
shape needed for later safe deallocation or growth.

Allocator ownership is why `String` is not a widened `StaticString`. A
`StaticString` can point at baked program bytes. A `String` must know which
allocator owns its runtime bytes:

```zen
String<A>: {
    ptr: RawPtr<u8>,
    len: usize,
    capacity: usize,
    allocator: A,
}

empty_string<A: Allocator<u8, Sync>> =
    (allocator: A, capacity: usize) Result<String<A>, AllocError> {
    allocator.alloc(capacity) ?
        | Ok(ptr) {
            Result<String<A>, AllocError>.Ok(String<A> {
                ptr: ptr,
                len: 0,
                capacity: capacity,
                allocator: allocator
            })
        }
        | Err(error) { Result<String<A>, AllocError>.Err(error) }
}
```

The intended call shape stays ordinary Zen:

```zen
work = (allocator: Allocator<u8, Sync>) Result<RawPtr<u8>, AllocError> {
    allocator.alloc(64)
}
```

The effect mode is part of the allocator capability, so the type system can
distinguish synchronous allocation from task-returning asynchronous allocation.

A dynamic `String` follows the same ownership rule as `Buffer`: it is not just
bytes, it is bytes plus allocator ownership and an effect mode. In other words,
`StaticString` is a compile-time program value, while `String` is a runtime
owned allocation.

Read allocator signatures from left to right:

- `Allocator<T, Sync>` can allocate `T` now and returns `Result<..., E>`.
- `Allocator<T, Async>` can allocate `T` later and returns
  `Task<Result<..., E>>`.
- `Buffer<T, A>` owns memory only because `A` is kept with the buffer.
- `String` is the text-shaped version of the same rule: owned bytes plus an
  allocator capability, not a widened `StaticString`.

### Ownership Preview

The ownership rule matches the string model above: static data is a baked,
non-owning value, while dynamic storage is allocator-backed. A dynamic buffer
keeps both the pointer and the allocator capability that owns the allocation.
That shape makes allocation, deallocation, and effect mode visible in the type
instead of hiding it behind a literal or method call.

Related gated previews:

- comptime type matching works on typed metadata, not runtime values, and stays
  gated until the metadata path is promoted.
- actor framework APIs live in `std` first; Zen has no stable actor syntax yet.
- host syscalls require explicit host-effect declarations before promotion.

For the current contract and gate status, see `docs/V1_SPEC.md`.

## Tooling JSON

Zen exposes compiler-owned JSON for tools and agents:

```sh
zen emit-json ast main.zen
zen emit-json symbols main.zen
zen emit-json typed main.zen
zen emit-json diagnostics main.zen
zen emit-json layout main.zen
zen emit-json hir main.zen
zen emit-json mir main.zen
zen emit-json target-yaml target.yaml
```

`ast` is an unchecked parse view. `symbols` is resolver output, `typed` is the
checked typed program, and `diagnostics` is machine-readable error output.
`layout` reports checked ABI layout facts for the stable subset, including
primitive sizes, `StaticString`, pointer-sized views, and struct field offsets.
`hir` reports checked declaration-level HIR for tools and agents that need a
stable graph of types, functions, and globals without owning compiler truth.
`mir` reports checked minimal function/block MIR for tools that need a stable
control-flow summary while broader MIR lowering is still being promoted.
`target-yaml` validates a human-authored target description and current C
backend options into canonical `zen.target.v0` JSON while rejecting attempts to
override compiler-owned type layouts or select unsupported backends.

Hand-authored JSON is not accepted as compiler truth; the compiler emits these
views from source so tools cannot override checked types or layouts.

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
