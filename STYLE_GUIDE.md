# Style Guide

## Loop Syntax

- **DO NOT EVER implement `in`-based loop syntax.**
  - No `for i in ...`, no `loop ... in ...`.
  - This is a permanent design decision.
- Use explicit, non-dangling, non-tertiary forms only.
- Prefer:
  - `loop 0..10 { ... }` (range as condition, no variable)
  - Or C-style: `loop i = 0; i < 10; i += 1 { ... }` 

## Variable Declaration Philosophy

- **No 'let', 'var', or 'const' prewords.**
  - Variable declarations use symbolic, explicit, and minimal forms only.
  - Use `:=` for immutable, `::=` for mutable.
  - No redundant prewords anywhere in the language.
  - Example:
    - `x := 42` (immutable)
    - `y ::= 10` (mutable) 