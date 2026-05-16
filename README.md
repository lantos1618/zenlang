# Zen

Zen is a systems language for code that should stay readable when programs get
large: declarations are compact, data shapes are explicit, and control flow is
driven by expressions and pattern matching instead of statement-heavy ceremony.

```zen
{ io } = std

Result<T, E>:
    Ok(T),
    Err(E)

Point: {
    x: i32,
    y: i32,
}

Point.sum = (self: Point) i32 {
    self.x + self.y
}

main = () i32 {
    p = Point { x: 20, y: 22 }

    p.sum() == 42 ?
        | true { io.println("the point adds up") }
        | false { io.println("try another point") }

    0
}
```

## Why Zen

- Prefix-first declarations keep functions, structs, enums, imports, methods,
  and behaviors visually regular.
- Pattern matching is the core branching form, so booleans, enums, results, and
  options all use the same `?` shape.
- Algebraic data types make absence and failure explicit with `Option<T>` and
  `Result<T, E>` instead of nulls and hidden exceptions.
- Dot calls work for ordinary functions and declared methods, which keeps APIs
  fluent without requiring class-based design.
- Behaviors describe required capabilities directly, so generic code can state
  the operations it needs.
- The language is designed for predictable native programs: explicit types,
  explicit imports, and no hidden object model.

## Language Shape

```zen
{ io } = std

Option<T>:
    None,
    Some(T)

unwrap_or<T> = (value: Option<T>, fallback: T) T {
    value ?
        | Some(inner) { inner }
        | None { fallback }
}

Display: behavior {
    display: (Self) str
}
```

Zen code is meant to read from the outside in:

- imports bind names from modules with destructuring syntax;
- data types are named first, then shaped with fields or variants;
- functions put the name first, then parameters, result type, and body;
- methods attach behavior to a type with `Type.method`;
- generic constraints use behavior bounds such as `T: Display`.

## Learn

- [Learn Zen In Y Minutes](docs/learn_zen_in_y_minutes.md)
- [Examples](examples/README.md)
- [V1 language contract](docs/V1_SPEC.md)
- [Phase plan](docs/PHASE_PLAN.md)
- [Completion audit](docs/COMPLETION_AUDIT.md)

The README is intentionally about the language. Current implementation status,
gates, and audit details live in the docs linked above.
