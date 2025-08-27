# Style Guide

## Loop Syntax

- **DO NOT EVER implement `in`-based loop syntax.**
  - No `for i in ...`, no `loop ... in ...`.
  - This is a permanent design decision.
- Use explicit, non-dangling, non-tertiary forms only.
- Prefer functional iteration:
  - `range(0, 10).loop(i -> { ... })` (functional range iteration)
  - `loop condition { ... }` (while-like)
  - `loop { ... }` (infinite)

## Variable Declaration Philosophy

- **No 'let', 'var', or 'const' prewords.**
  - Variable declarations use symbolic, explicit, and minimal forms only.
  - Use `:=` for immutable, `::=` for mutable.
  - No redundant prewords anywhere in the language.
  - Example:
    - `x := 42` (immutable)
    - `y ::= 10` (mutable) 

## Conditionals
- DO NOT EVER IMPLEMENT `if`, `match` `switch` `case` `of`