# Learn Zen In Y Minutes

Zen is a systems language for explicit programs: prefix-first declarations,
plain data shapes, pattern matching, generics, behaviors, visible ownership,
and predictable native output.

Runnable examples live in `examples/` and `tests/zen/`. This page is the
language tour: copy the stable source forms you can use in examples today, and
read the gated design previews that show intended syntax for features that are
still behind diagnostics.

Stable Zen avoids hidden allocation, exceptions, null, `break`, `continue`,
and keyword exits. Values come from final expressions. Loops use explicit
control calls. Heap ownership appears in the type/API surface. The guide below
teaches the canonical source spelling.

If you only read one page, read the sections through `Learn It In One Pass`.
They give the public spelling for functions, final expressions, static text,
dynamic text previews, loops, sync/async effect previews, and allocator-backed
ownership. The rest of the guide expands those same rules with more examples.

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
- `StaticString` is baked into the program as static bytes plus length; it is a
  static pointer-and-length view, not allocator-backed String or other owned
  dynamic text;
- allocator-backed `String<A>` is owned runtime memory and must carry the
  allocator that can grow or release it;
- sync, async, allocator, raw-memory, actor, and comptime type-matching
  surfaces are gated design work.

The smallest useful rule set is:

- write declarations in prefix form: `name = ...`, `Type.method = ...`,
  `Type.implements(Behavior)`;
- use final expressions for values; Zen does not use a `return` keyword;
- use `loop((l) { ... })`, then `l.next()` or `l.done()`;
- use `StaticString` for literal text baked into the program;
- use allocator-backed `String<A>` only when runtime-owned text is intended;
- make sync work produce checked values now, and async work produce `Task<...>`;
- keep allocators in the owner type that can grow or release heap storage.

## The Five Rules To Remember

1. A block result is its final expression. Zen does not use `return`.
2. A string literal is `StaticString`: static bytes plus length baked into the
   program image. It is a pointer-and-length view and does not allocate.
3. Runtime-owned text is `String<A>` or another allocator-backed owner. If it
   can grow, shrink, or release memory, the allocator belongs in the type.
4. Loops have one stable shape: `loop((l) { ... })`, with explicit `l.next()`,
   `l.done()`, `next(l)`, or `done(l)` control edges.
5. Sync and async are visible in types. Sync returns a checked value now;
   async returns task-shaped work such as `Task<Result<T, E>>`.

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
the corresponding subsystem is implemented.

The rule for this document is simple: stable examples should compile. Preview
examples make the future syntax concrete and are labeled as previews.

## Learn It In One Pass

This is the shortest useful version of the language:

| Concept | Copy this shape |
| --- | --- |
| Static text | `name: StaticString = "Zen"` |
| Dynamic text preview | `String<A>` where `A` owns growth and release |
| Function result | put the value at the end of the block |
| Counted loop | `loop((l) { ... l.next() ... l.done() ... })` |
| Nested loop exit | call the outer handle, such as `outer.done()` |
| UFC loop control | `done(l)` and `next(l)` |
| Sync effect preview | produce `Result<T, E>` directly |
| Async effect preview | produce `Task<Result<T, E>>` |
| Sync allocator preview | `Allocator<T, Sync>` returns allocation results now |
| Async allocator preview | `Allocator<T, Async>` returns task-shaped allocation results |

Stable code should look like this:

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

The matching preview shape for allocator-backed work is:

```zen
make_now<T, A: Allocator<T, Sync>> = (allocator: A, len: usize) Result<RawPtr<T>, AllocError> {
    allocator.alloc(len)
}

make_later<T, A: Allocator<T, Async>> = (allocator: A, len: usize) Task<Result<RawPtr<T>, AllocError>> {
    allocator.alloc(len)
}
```

Read the outer type first. `Result<...>` means the checked value is available
now. `Task<Result<...>>` means the checked value belongs to scheduled work.
`StaticString` is static program storage; `String<A>` is dynamic owned storage
and must keep allocator ownership visible.

### Copy-Paste Loop Forms

Loops have one public entry point. Start with `loop((label) { ... })`, then
make every edge explicit with `label.next()` or `label.done()`.

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

Nested loops name the loop they want to control. Exiting an outer loop is not
a special keyword; it is just an explicit call on the outer handle.

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

UFC is the same control operation with the verb first:

```zen
loop((l) {
    done(l)
    next(l)
})
```

Do not translate these to `while`, `for`, `break`, `continue`, or a body-first
loop. Those are not the public Zen forms.

### Copy-Paste Text, Sync, Async, And Allocator Forms

Use `StaticString` when the bytes are baked into the program. It is a static
pointer-and-length view:

```zen
title: StaticString = "Zen"
```

Use `String<A>` only when runtime text owns memory managed by allocator `A`:

```zen
String<A>: {
    ptr: RawPtr<u8>,
    len: usize,
    capacity: usize,
    allocator: A,
}
```

Sync APIs produce checked data directly. Async APIs produce task-shaped work:

```zen
read_now = (source: Source, allocator: Allocator<u8, Sync>) Result<Bytes<u8>, IoError> {
    source.read_all(allocator)
}

read_later = (source: Source, allocator: Allocator<u8, Async>) Task<Result<Bytes<u8>, IoError>> {
    source.read_all_async(allocator)
}
```

Allocator capabilities follow the same outer-type rule:

```zen
Allocator<T, Sync>: behavior {
    alloc: (Self, count: usize) Result<RawPtr<T>, AllocError>
}

Allocator<T, Async>: behavior {
    alloc: (Self, count: usize) Task<Result<RawPtr<T>, AllocError>>
}
```

The practical reading is:

| Surface | Meaning |
| --- | --- |
| `StaticString` | static bytes plus length baked into the program |
| `String<A>` | owned dynamic bytes plus allocator ownership |
| `Result<T, E>` | checked data is available now |
| `Task<Result<T, E>>` | checked data belongs to scheduled work |
| `Allocator<T, Sync>` | allocation returns now |
| `Allocator<T, Async>` | allocation returns as task-shaped work |

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

Preview examples are included only when the API shape matters to the language
model:

| Preview surface | Intended reading |
| --- | --- |
| `String<A>` | dynamic owned text carrying allocator ownership |
| `Allocator<T, Sync>` | allocation happens now and returns `Result<...>` |
| `Allocator<T, Async>` | allocation is task-shaped and returns `Task<Result<...>>` |
| `Task<T>` | async work is represented in the type instead of hidden in a call |
| `RawPtr<T>` and raw intrinsics | explicit low-level memory work, gated until ownership rules exist |

The split matters most for strings. `"hello"` is a `StaticString`: static
bytes plus length baked into the program image. It has a known location and
byte count after compilation. It is not a `String<A>`. A dynamic string owns
runtime memory, has capacity, can grow, and needs an allocator capability in
its type/API surface.

## Use This Mental Model

Zen keeps important edges visible:

- Control is explicit. Functions, matches, and blocks produce values from final
  expressions. Loop control is prefix-only: enter with `loop((l) { ... })`,
  then call `l.done()`, `l.next()`, `done(l)`, or `next(l)`.
  The phrase to remember is simple: loop control is prefix-only.
- Text ownership is explicit. StaticString is not a String. `StaticString` is
  not `String<A>`. Static text and dynamic text are different types, and a
  literal never silently allocates dynamic text.
- Effects are explicit. Sync work produces a direct checked value. Async work
  returns a task-shaped value.
- Allocation is explicit. Dynamic owners carry the allocator that can grow or
  release their storage.
- Behavior relationships are explicit. Use `Type.implements(Behavior)`,
  `Type.requires(Behavior)`, and `Child.extends(Parent)`.
- Tooling truth comes from the compiler. JSON views are emitted from source;
  hand-authored JSON is not accepted as checked program state.

Keyword-style forms from other languages are not stable tutorial syntax.
Translate them into receiver-first or prefix-first Zen. That means no
`impl ... for ...`, no `extends Behavior` keyword block, no `return`, and no
body-first loop.

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

## Read Static And Dynamic Text Correctly

Text in Zen has two different ownership shapes:

| Text shape | What it means |
| --- | --- |
| `StaticString` | static bytes plus length baked into the program image |
| `String<A>` | owned dynamic bytes whose storage is managed by allocator `A` |

`StaticString` is the type of string literals. The compiled program knows where
the bytes live and how long they are. Passing a `StaticString` copies that
pointer-and-length view; it does not allocate, grow, free, or transfer
ownership.

`String<A>` is a dynamic owner. It can be built at runtime, can have capacity,
and can grow, so the allocator must be visible in the type. A literal such as
`"Zen"` never silently becomes `String<A>`.

## Read Effects And Allocators Correctly

`Sync` and `Async` are effect modes in type surfaces. They are not function
keywords and they are not decorative marker names:

```zen
read_now = (source: Source, allocator: Allocator<u8, Sync>) Result<Bytes<u8>, IoError> {
    source.read_all(allocator)
}

read_later = (source: Source, allocator: Allocator<u8, Async>) Task<Result<Bytes<u8>, IoError>> {
    source.read_all_async(allocator)
}
```

The outer result type tells you whether work is complete. Sync work returns a
checked value now, such as `Result<T, E>`. Async work returns task-shaped work,
such as `Task<Result<T, E>>`. There is no source-level `async fn` syntax in the
stable tour.

Allocators follow the same rule. `Allocator<T, Sync>` and
`Allocator<T, Async>` are different capabilities because they allocate under
different effect modes. Any dynamic owner that stores heap memory must carry
the allocator that can later grow or release that memory.

## What Is Stable Right Now

Use this stable core for examples, tutorials, and small programs:

```zen
{ io } = std

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
    label: StaticString = "sum"
    io.println("${label}: ${sum_to(10)}")
    0
}
```

That program uses only stable shapes: imports, typed functions, immutable and
mutable locals, static text, final expressions, interpolation at an output
boundary, pattern matching, and prefix-only loops. It does not allocate a
dynamic string, start async work, call a raw intrinsic, or rely on an implicit
return.

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

Nested loops can control the current loop or an outer loop by naming the
handle:

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

Those calls are loop-control syntax, not ordinary methods named by strings.
The compiler recognizes the control operation for the loop handle; user code
does not implement `done` or `next`.

The UFC spelling puts the control verb first while still naming the handle:

```zen
loop((l) {
    done(l)
    next(l)
})
```

The important part is that every edge is visible. Continue the current loop
with `next`; exit it with `done`:

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
arbitrary user methods on a library object. The compiler recognizes only the
control verbs for the loop handle here; this is not general user-defined
method dispatch.

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
its fixed byte count with the value, so a literal can be passed around without
allocating or changing ownership. Its bytes live in the program image; the
value is a stable pointer-and-length view and does not own or free memory.
The location and byte count are known from the compiled program, so the value
cannot grow, shrink, or release memory.

The allocator-backed `String<A>` type is dynamic: it owns runtime memory and
carries allocator-managed capacity, length, and storage. It also carries
allocator ownership. It can grow, can be built at runtime, and must be created
through allocator-aware APIs once the allocator model is promoted.
Until that ownership path exists, source-level `String` annotations
are gated; use `StaticString` for literal/static text.

That distinction is deliberate:

- `StaticString` is a non-owning view into program storage.
- `StaticString` has a fixed byte count known from the compiled program.
- Copying or passing `StaticString` copies the view, not a heap allocation.
- `String<A>` is owned dynamic memory and therefore needs allocator ownership.
- A literal such as `"Zen"` does not allocate a dynamic `String`.
- Runtime text construction belongs on an allocator-aware path.
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
context or returns task-shaped async work explicitly. They are not source
keywords placed before a function; they appear in the types that describe the
capability being used.

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

The allocator type parameter is not decoration. It answers three questions at
the call boundary: what element type is being allocated, whether the work is
sync or async, and which capability must later release or grow the storage.

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
