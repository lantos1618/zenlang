# Learn Zen In Y Minutes

Zen is a small work-in-progress systems language. This guide uses examples that
match the current tested compiler surface in this repository.

## Hello

```zen
{ io } = std

main = () i32 {
    io.println("hello")
    return 0
}
```

Top-level declarations use prefix-style forms:

- imports: `{ io } = std`
- functions: `name = (...) ReturnType { ... }`
- structs: `Name: { ... }`
- enums: `Name: Variant, Variant(Payload)`
- behaviors: `Name: behavior { ... }`

## Values

```zen
main = () i32 {
    answer = 42
    message = "zen"
    ok = true
    return answer
}
```

Local bindings are immutable by default. Use explicit mutable forms only where
the current compiler tests cover them.

## Functions

```zen
add = (a: i32, b: i32) i32 {
    return a + b
}

main = () i32 {
    return add(20, 22)
}
```

Functions are expressions with typed parameters and an explicit return type.

## Structs

```zen
{ io } = std

Person: {
    name: StaticString,
    age: i32,
}

main = () i32 {
    p = Person {
        name: "Alice",
        age: 30
    }

    io.println("${p.name}")
    io.println("${p.age}")
    return 0
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

The `?` operator is the main pattern-match form. Enum matches are checked for
exhaustiveness and duplicate arms in the current compiler.

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

Generic enum variants are constructed with the specialized enum name:

```zen
some = Option<i32>.Some(42)
none = Option<i32>.None
```

## Methods

```zen
Point: {
    x: i32,
    y: i32,
}

Point.sum = (self: Point) i32 {
    return self.x + self.y
}

main = () i32 {
    p = Point { x: 10, y: 32 }
    return p.sum()
}
```

Methods are declared as `Type.method = (...) ReturnType { ... }`. Calls use
normal dot syntax.

## Generics

```zen
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    return self.value
}

main = () i32 {
    box = Box<i32> { value: 42 }
    return box.get()
}
```

The current compiler monomorphizes reachable generic functions, structs, enums,
and methods into concrete generated C symbols.

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

Nested generic types are written directly:

```zen
value = Result<Option<i32>, str>.Ok(Option<i32>.Some(1))
```

## Behaviors

```zen
Json: behavior {
    to_json: (Self) str
}

Point: {
    x: i32
}

Point.implements(Json) {
    to_json = (self: Point) str {
        return "point"
    }
}

encode<T: Json> = (value: T) str {
    return value.to_json()
}
```

Behaviors describe required methods. Generic functions can use behavior bounds
with `T: BehaviorName`.

## Imports

```zen
{ io } = std
```

Imports use destructuring-style binding from a module path. Multi-file examples
live under `tests/zen/multi_file_*`.

## What Is Still Not Stable

These are design goals, not stable language promises yet:

- full `build.zen` execution
- typed allocator semantics
- Sync/Async effect checking
- actor runtime integration
- JSON/YAML IR and config boundaries
- package manager and formatter

For the current contract, see `docs/V1_SPEC.md`. For runnable examples, see
`tests/zen/` and `examples/project/`.
