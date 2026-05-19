# Learn Zen In Y Minutes

Zen is a systems language built around prefix-first declarations, explicit data
shapes, pattern matching, generics, behaviors, visible ownership, and
predictable native output.

Runnable examples live in `examples/` and `tests/zen/`. This guide is a fast
tour of the source shape: what stable examples should look like today, and how
the reserved preview surfaces are intended to read once promoted.

The guide has two layers:

- stable source forms you can use in examples today;
- gated design previews that show intended syntax, but currently compile to
  feature-gate diagnostics instead of pretending the feature exists.

Stable Zen avoids hidden allocation, exceptions, null, `break`, `continue`,
and keyword exits. Values come from final expressions. Loops use explicit
control calls. Heap ownership has to appear in the type/API surface. The
guide below teaches the canonical source spelling, not transitional aliases.

Quick map:

- declarations are prefix-first: the name appears before the operation;
- values come from final expressions, not `return`;
- branching uses `?` pattern matching for bools, enums, `Option`, and `Result`;
- loops use one prefix form, `loop((label) { ... })`;
- loop exits are `label.done()` and `label.next()`, with `done(label)` and
  `next(label)` as UFC spellings;
- type relationships are receiver-first declarations such as
  `Point.implements(Json)`, `PrettyJson.extends(Json)`, and
  `Point.requires(Json)`;
- `StaticString` is baked into the program; it is not allocator-backed String
  or other dynamic text;
- allocator-backed `String<A>` is owned runtime memory and must carry the
  allocator that can grow or release it;
- sync, async, allocator, raw-memory, actor, and comptime type-matching
  surfaces are gated design work until promoted.

## The Whole Shape In One Page

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

That example is the core of Zen: prefix declarations, typed data, attached
methods, behavior implementations, bounded generics, expression-oriented
control flow, and explicit loop control.

## Read This Guide In Two Passes

Use the stable sections for code you put in public examples today. Those
sections cover imports, values, functions, methods, structs, enums, matching,
generics, behaviors, receiver-first behavior relationships, modules, defer,
and prefix-only loops.

Use the gated-preview sections to understand the intended language shape for
memory ownership, dynamic strings, sync/async effects, allocators, raw memory,
actors, and comptime type matching. Preview examples are still Zen-shaped, but
the compiler should reject them with explicit feature-gate diagnostics until
the corresponding subsystem is promoted.

The rule for this document is simple: stable examples should compile; preview
examples should make the future syntax concrete without pretending it is
stable.

## Stable Vs Preview Surface

Stable examples are the forms to copy into runnable source today:

| Stable today | Why it is stable |
| --- | --- |
| `StaticString` literals | static bytes are baked into the program and need no allocator |
| final expressions | every value-producing block has one obvious result |
| `value ? | Pattern { ... }` | bools, enums, `Option`, and `Result` share one branching shape |
| `loop((l) { ... })` | loops have explicit state and explicit control edges |
| `l.done()`, `l.next()`, `done(l)`, `next(l)` | loop control is visible and prefix/UFC-friendly |
| `Type.method = ...` | methods are attached functions with an explicit receiver |
| `Type.implements(Behavior)` | behavior relationships keep the changed type on the left |

Preview examples are included only when the future API shape matters to the
language model:

| Preview surface | Intended reading |
| --- | --- |
| `String<A>` | dynamic owned text carrying allocator ownership |
| `Allocator<T, Sync>` | allocation happens now and returns `Result<...>` |
| `Allocator<T, Async>` | allocation is task-shaped and returns `Task<Result<...>>` |
| `Task<T>` | async work is represented in the type instead of hidden in a call |
| `RawPtr<T>` and raw intrinsics | explicit low-level memory work, gated until ownership rules are promoted |

The split matters most for strings. `"hello"` is a `StaticString`: static
storage plus length baked into the program. It is not a `String<A>`, because a
dynamic string owns runtime memory, has capacity, can grow, and needs an
allocator capability in its type/API surface.

## Use This Mental Model

Zen keeps important edges visible:

- Control is explicit. Functions, matches, and blocks produce values from final
  expressions. loop control is prefix-only: enter with `loop((l) { ... })`,
  then call `l.done()`, `l.next()`, `done(l)`, or `next(l)`.
- Text ownership is explicit. StaticString is not a String. `StaticString` is
  not `String<A>`. Static text and dynamic text are different types.
- Effects are explicit. Sync work produces a direct checked value. Async work
  returns a task-shaped value.
- Allocation is explicit. Dynamic owners carry the allocator that can grow or
  release their storage.
- Behavior relationships are explicit. Use `Type.implements(Behavior)`,
  `Type.requires(Behavior)`, and `Child.extends(Parent)`.
- Tooling truth comes from the compiler. JSON views are emitted from source;
  hand-authored JSON is not accepted as checked program state.

Transitional keyword phrases are not part of stable tutorial syntax. If a form
reads like a borrowed phrase from another language, translate it into the
receiver-first or prefix-first Zen form. That means no `impl ... for ...`,
no `extends Behavior` keyword block, no `return`, and no body-first loop.

## Translation Cheat Sheet

| If you reach for | Use |
| --- | --- |
| `return` plus a value | put the value in the final expression |
| `while condition { ... }` | `loop((l) { condition ? | true { ... l.next() } | false { l.done() } })` |
| `for item in items { ... }` | explicit state plus `loop((l) { ... })` |
| `break` | `l.done()` or `done(l)` |
| `continue` | `l.next()` or `next(l)` |
| `impl Type for Behavior` | `Type.implements(Behavior) { ... }` |
| `extends Parent` keyword block | `Child.extends(Parent)` |
| `requires Behavior` keyword block | `Type.requires(Behavior)` |
| `async fn` | a function whose type is `Task<Result<T, E>>` or another task-shaped type |
| string literal text | `StaticString` |
| growable owned text | `String<A>` or another owner that carries allocator ownership |

## Copy These Forms First

Zen is prefix-first at declaration and control boundaries. Stable Zen is deliberately small; most code is built from this surface:

| Need | Spell it like this |
| --- | --- |
| Import | `{ io } = std` |
| Immutable local | `name = value` |
| Mutable inferred local | `name ::= value` |
| Reassignment | `name = new_value` |
| Typed local | `name: Type = value` |
| Function | `name = (arg: Type) ResultType { expr }` |
| Method | `Type.method = (self: Type) ResultType { expr }` |
| Struct | `Name: { field: Type }` |
| Enum | `Name: Variant, Variant(Payload)` |
| Match | `value ? | Pattern { expr }` |
| Loop | `loop((l) { ... l.next() ... l.done() ... })` |
| Behavior | `Name: behavior { method: (Self) Type }` |
| Implementation | `Type.implements(Behavior) { ... }` |
| Required behavior | `Type.requires(Behavior)` |
| Behavior inheritance | `ChildBehavior.extends(ParentBehavior)` |
| Generic bound | `name<T: Behavior> = (...) T { ... }` |

```zen
{ io } = std

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

main = () i32 {
    label: StaticString = "baked into the program"
    point = Point { x: 20, y: 22 }
    io.println("${label}")
    point.sum()
}
```

If a feature needs hidden allocation, implicit scheduling, exceptions, null, or
statement-level early exits, it is not part of the stable shape. Zen makes
those things visible with data types, final expressions, loop-control calls,
and allocator or effect parameters.

There is no `impl Type for Behavior` spelling. There is no source-level
`async` keyword. There is no `async fn` spelling. Sync, async, and allocator
behavior is visible through types such as `Allocator<T, Sync>`,
`Allocator<T, Async>`, `Result<T, E>`, and `Task<Result<T, E>>`.

Behavior declarations also stay receiver-first. Use:

```zen
Type.implements(Behavior) { ... }
ChildBehavior.extends(ParentBehavior)
Type.requires(Behavior)
```

Those forms keep the thing being changed on the left. There is no separate
`impl` keyword, no `extend` keyword, and no hidden trait-relationship syntax.

## Control Flow At A Glance

| Need | Stable form |
| --- | --- |
| Choose by bool, enum, `Option`, or `Result` | `value ? | Pattern { expr }` |
| Produce a function result | put the value in the final expression |
| Repeat work | `loop((l) { ... })` |
| Continue a loop | `l.next()` or `next(l)` |
| Exit a loop | `l.done()` or `done(l)` |
| Fail or be absent | use `Result<T, E>` or `Option<T>` |

No alternate loop syntax exists. There is no `while (...) { ... }`, no
`for item in items { ... }`, and no body-first loop spelling. Convert those
forms to `loop((l) { ... })` with explicit state and explicit `done`/`next`
edges.

Loop control recipes:

```zen
// Continue the current loop.
loop((l) {
    should_continue ?
        | true { l.next() }
        | false { l.done() }
})

// UFC spelling for the same control verbs.
loop((l) {
    finished ?
        | true { done(l) }
        | false { next(l) }
})

// Nested loops can exit an outer loop directly.
loop((outer) {
    loop((inner) {
        stop ?
            | true { outer.done() }
            | false { inner.next() }
    })

    outer.next()
})
```

Those calls are loop-control syntax, not ordinary methods named by strings.
The compiler recognizes the control operation for the loop handle; user code
does not implement `done` or `next`.

The important part is that every edge is visible:

```zen
count_down = (start: i32) i32 {
    current ::= start

    loop((l) {
        current == 0 ?
            | true { l.done() }
            | false {
                current = current - 1
                l.next()
            }
    })

    current
}
```

That same rule replaces both `break` and `continue`:

```zen
skip_until_positive = (value: i32) i32 {
    seen ::= 0

    loop((l) {
        value > 0 ?
            | true { l.done() }
            | false {
                seen = seen + 1
                l.next()
            }
    })

    seen
}
```

The core syntax to keep in your head:

```zen
{ name } = module.path

Name: { field: Type }

Name: Variant, Variant(Payload)

function = (arg: Type) ResultType {
    final_expression
}

Type.method = (self: Type) ResultType {
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

## The Shape

Zen declarations start with the thing being introduced or changed:

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

The common forms are imports, structs, enums, functions, attached methods,
behaviors, and receiver-first behavior relationships.

## Hello

```zen
{ io } = std

main = () i32 {
    io.println("hello")
    0
}
```

Top-level declarations use prefix-first declarations: imports, functions,
structs, enums, methods, and behaviors.

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

Comments are not part of semantic JSON.

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
`cast(value, Type)` syntax so type-changing operations stay visible.

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
Use `void` for functions that only perform effects.

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

Attached functions can be called with `value.method(args)` dot syntax or
`method(value, args)` uniform function call
syntax. Dot syntax keeps the receiver first when the operation reads like a
method. UFC keeps the operation first when that is clearer. These are
call-site
spellings, not alternate declaration forms. The receiver is still an explicit
argument in the declared function type.

## Blocks Produce Their Final Expression

```zen
max = (a: i32, b: i32) i32 {
    a > b ?
        | true { a }
        | false { b }
}
```

Zen does not use a `return` keyword. Function bodies, match arms, and nested
blocks produce values from their final expression. Pattern arms are blocks, so
nested decisions stay expression-oriented.

## Structs

```zen
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
```

Nested generic types are written directly, such as
`Result<Option<i32>, StaticString>`.

## Error Handling

Zen models absence and failure with ordinary data. No exceptions are thrown
behind a call boundary. No null value exists to check at runtime.

```zen
divide = (a: f64, b: f64) Result<f64, StaticString> {
    b == 0.0 ?
        | true { Result<f64, StaticString>.Err("division by zero") }
        | false { Result<f64, StaticString>.Ok(a / b) }
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
```

Methods are declared as `Type.method = (...) ResultType { ... }`. Calls use dot
syntax. A method is still a function with an explicit receiver.

## Attached Methods

```zen
Counter: {
    value: i32,
}

Counter.inc = (self: Counter) Counter {
    Counter { value: self.value + 1 }
}
```

Prefer direct `Type.method = ...` declarations in public examples. They keep
the receiver type on the left, avoid extra grouping syntax, and line up with
dot and UFC calls.

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
```

The compiler monomorphizes reachable generic functions, structs, enums, and
methods into concrete generated C symbols. Behavior bounds can appear on generic parameters when a generic function needs a capability:

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
```

Behaviors describe required methods. Generic functions can use behavior bounds
with `T: BehaviorName`.

## Behavior Inheritance And Requires

```zen
PrettyJson.extends(Json)
Point.requires(Json)
```

These are receiver-first relationship declarations, not free-floating
keywords. Zen keeps the left-hand side as the thing being changed:

| Relationship | Meaning |
| --- | --- |
| `Point.implements(Json)` | `Point` provides `Json` methods |
| `PrettyJson.extends(Json)` | `PrettyJson` includes `Json` requirements |
| `Point.requires(Json)` | `Point` is required to have `Json` available |

## Loops

Zen has one loop form. There are no `for` or `while` keywords, and loop exits
are explicit calls instead of `break` or `continue`. The spelling is
prefix-only: call `loop`, pass a loop-control parameter, then choose the next
edge with `done` or `next`.

The loop parameter is a control handle, not a user-defined object. `done` and
`next` are compiler-owned loop-control verbs for that handle. They are not
arbitrary user methods on a library object. The compiler recognizes only the control verbs for the loop handle here; this is not general user-defined method dispatch.

Counted loops use explicit state:

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

Loop syntax is prefix-only. `break` and `continue` are not Zen source forms.
Use `l.done()` and `l.next()` or their UFC forms instead. See
`examples/05_loops.zen` for the runnable tutorial version.

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

UFC loop control keeps the operation first while still naming the handle:

```zen
step = (done_now: bool) void {
    loop((l) {
        done_now ?
            | true { done(l) }
            | false { next(l) }
    })
}
```

The UFC spelling is still loop-control syntax. It does not introduce ordinary
functions named `done` or `next`:

```zen
loop((l) {
    finished ?
        | true { done(l) }
        | false { next(l) }
})
```

The complete stable loop surface is:

- `loop((l) { ... })` starts a loop and binds a control handle.
- `l.done()` exits that loop.
- `l.next()` continues that loop.
- `done(l)` and `next(l)` are the equivalent UFC forms.
- A nested loop can control an outer loop with `outer.done()` or `done(outer)`.
- There is no suffix/body-first loop spelling; `loop(...)` is the prefix entry
  point.

There is no hidden loop result channel. Accumulated values live in explicit
mutable bindings outside the loop and are read after `done`.

This is the complete mental rewrite:

```zen
// Source idea: repeat while i is not past limit.
loop((l) {
    i > limit ?
        | true { l.done() }
        | false {
            total = total + i
            i = i + 1
            l.next()
        }
})
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

## Memory And Ownership

Allocation is explicit. The stable subset does not hide
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

Owned dynamic storage is modeled with allocator-aware types once typed
allocators are promoted. Until then, the compiler rejects allocator-backed
source types instead of pretending allocation is free.

```zen
OwnedBytes<T, A>: {
    ptr: RawPtr<T>,
    len: usize,
    capacity: usize,
    allocator: A,
}
```

The practical rule is simple: if a value needs heap memory, the API must show
the allocator path. The pointer, length, and allocator capability travel
together.

That rule also explains the stable string split:

```zen
static_label = (label: StaticString) StaticString {
    label
}
```

`static_label("Zen")` passes static program text. A future dynamic-text API
should not accept that by accident as an owned string; it should ask for an
allocator-backed owner explicitly:

```zen
dynamic_label<A> = (label: String<A>) String<A> {
    label
}
```

The second signature is preview-only today, but it shows the ownership
contract: the caller is passing owned runtime text, and the allocator relation
is part of the type.

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
slice, and fixed-array shapes. raw pointer offset, casts, integer conversion,
load, store, atomics, raw syscalls, comptime type matching, and actor framework
surfaces are gated design work until layout, ownership, effects, and runtime
contracts are promoted.

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
its fixed length with the value, so a literal can be passed around without
allocating or changing ownership. Its bytes live in the program image; the
value is a stable pointer-and-length view and does not own or free memory.
The location and byte count are known from the compiled program, so the value
cannot grow.

The allocator-backed `String<A>` type is dynamic: it owns memory, carries
allocator-managed capacity, length, and storage. It also carries allocator
ownership. It can grow, can be built at runtime, and must be created through
allocator-aware APIs once the allocator model is promoted.
Until that ownership path exists, source-level `String` annotations
are gated; use `StaticString` for literal/static text.

That distinction is deliberate:

- `StaticString` is a non-owning view into program storage.
- `String<A>` is owned dynamic memory and therefore needs allocator ownership.
- A literal such as `"Zen"` does not allocate a dynamic `String`.
- String interpolation is non-owning in stable examples; only literal bytes are
guaranteed to be baked into program storage.

```zen
String<A>: {
    ptr: RawPtr<u8>,
    len: usize,
    capacity: usize,
    allocator: A,
}
```

APIs should say which kind of text they accept:

```zen
identity_static = (message: StaticString) StaticString {
    message
}

identity_dynamic<A> = (message: String<A>) String<A> {
    message
}
```

The first function accepts baked program text. The second accepts owned text
whose storage is managed by `A`; that allocator relationship is part of the
type instead of being hidden behind a plain `string`.

## Gated Preview: Sync, Async, And Allocators

The following syntax and APIs are gated design goals, not stable compiler
behavior yet. They are included here because they are central to the intended
language shape: allocation is explicit, async work is effect-aware, and sync
code cannot accidentally call async operations. Current compiler paths reject
these spellings with feature-gate diagnostics instead of treating them as
ordinary unknown names.

Read every example in this section as a preview. The syntax is intentionally
Zen-shaped, but `Sync`, `Async`, `Allocator`, `String<A>`, `Task<T>`, raw
allocation intrinsics, actor framework types, and scheduler operations are not
stable source yet.

### Sync/Async/Allocator Quick Rules

- `Sync` APIs compute now and produce direct checked data.
- `Async` APIs describe later work and produce a task-shaped value.
- `Allocator<T, Sync>` can allocate `T` now.
- `Allocator<T, Async>` can allocate `T` later.
- `Allocator<T, Sync>` allocates now and returns `Result<RawPtr<T>, AllocError>`.
- `Allocator<T, Async>` allocates later and returns
  `Task<Result<RawPtr<T>, AllocError>>`.
- allocator-backed owners keep the allocator with the pointer, length, and
  capacity facts.
- `StaticString` does not become `String<A>` by assignment or inference.
- `String<A>` owns dynamic bytes and therefore needs allocator ownership.
- async work returns a task-shaped value instead of hiding scheduler work inside
  an ordinary result.
- loop handles are compiler-owned; their control verbs are `done` and `next`,
  not arbitrary user methods.

In short:

```zen
// Sync: caller receives the checked value now.
make_now<T, A: Allocator<T, Sync>> = (allocator: A, len: usize) Result<RawPtr<T>, AllocError> {
    allocator.alloc(len)
}

// Async: caller receives a task that may later produce the checked value.
make_later<T, A: Allocator<T, Async>> = (allocator: A, len: usize) Task<Result<RawPtr<T>, AllocError>> {
    allocator.alloc(len)
}
```

There is no hidden conversion between those two shapes. The outer type is the
effect boundary.

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

The call site stays honest about timing:

```zen
load_config = (source: Source, allocator: Allocator<u8, Sync>) Result<Bytes<u8>, IoError> {
    read_now(source, allocator)
}

start_config_load = (source: Source, allocator: Allocator<u8, Async>) Task<Result<Bytes<u8>, IoError>> {
    read_later(source, allocator)
}
```

The intended source rule is:

- a `Sync` API returns the result it computed now;
- an `Async` API returns a `Task<...>` that represents work to run later;
- sync code can call sync code directly;
- Sync code cannot call async operations without an explicit runtime boundary;
- allocator and scheduler APIs should expose their effect mode in the type
  surface instead of hiding it behind a normal call.

There is no source-level `async` keyword in the stable tour. The preview keeps
the effect in ordinary Zen types: `Sync`, `Async`, `Task<T>`, and allocator
capabilities.

Planned `.await()` and scheduler APIs are gated until task lowering and effect
checking are promoted.

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
    capacity: usize,
    allocator: A,
}
```

Sync allocation returns `Result` directly. Async allocation returns
`Task<Result<...>>`. Dynamic memory ownership is visible in the returned type:
bytes plus allocator ownership and an effect mode.

Allocator-backed construction carries the allocator through the resulting
owner:

```zen
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

`Buffer<T, A>` owns memory only because `A` is kept with the buffer. Passing a
raw pointer alone is just an address.

An async allocator has the same ownership goal, but its first result is a task
for allocation work, not an allocated owner:

```zen
allocate_later<T, A: Allocator<T, Async>> = (allocator: A, len: usize) Task<Result<RawPtr<T>, AllocError>> {
    allocator.alloc(len)
}
```

The important distinction is the outer type. Sync allocation gives back
`Result<...>` now. Async allocation gives back `Task<Result<...>>`, so callers
cannot confuse scheduled work with completed allocation. Building an owned
`Buffer<T, A>` from that async result belongs at an explicit scheduler/task
boundary once async lowering is promoted.

Raw allocation intrinsics such as `@builtin.raw_allocate(...)`,
`@builtin.raw_deallocate(...)`, and `@builtin.raw_reallocate(...)` are gated.
They exist as compiler-owned names so allocator diagnostics can be specific,
but stable source code should not call them yet.

### Ownership Preview

The allocator is part of the owner. A buffer or dynamic string is not just a
pointer and a length; it must also carry the capability that can release or
grow that storage.

```zen
Bytes<T, A>: {
    ptr: RawPtr<T>,
    len: usize,
    allocator: A,
}
```

This is why `String<A>` is not a widened `StaticString`: static text points at
baked program bytes, while dynamic text owns runtime bytes and must carry the
allocator that owns those bytes.

## Tooling JSON

Compiler tooling output is generated from checked source. Hand-authored JSON is
not accepted as a substitute for source input.

The useful rule for agents and editors is: compile source, then ask the
compiler for diagnostics, HIR, IR, or layout JSON. Treat those JSON views as
derived facts, not as editable program state.

## More To Read

- `README.md` for the language pitch.
- `examples/README.md` for canonical runnable examples.
- `docs/V1_SPEC.md` for the full v1 contract and gated design inventory.
- `docs/PHASE_PLAN.md` for implementation sequencing.
- `docs/COMPLETION_AUDIT.md` for audit details.
