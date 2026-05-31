# Async plan

The `@async`/`@await` keyword model — `Future<T>`, the state-machine transform,
the per-frame `@builtin.poll` driver, and the `block_on`/`Scheduler` stdlib built
on them — has been **removed** from the compiler, stdlib, and tests. It coupled
async to compiler machinery (a function-coloring red/blue split, a fragile
linear-body lowering) and never reached a usable surface.

## New direction (separate phase — not yet built)

Async will be reimplemented as **stackful coroutines** over libc `ucontext`
(`getcontext`/`makecontext`/`swapcontext`, reached through `@extern`), driven by
`Sync`/`Async` allocators:

- **No function coloring** — no red/blue problem; any function can suspend.
- **Pure stdlib** — coroutine create/resume/yield and the scheduler are ordinary
  Zen over `@extern` libc calls plus the existing memory intrinsics.
- **Zero compiler async machinery** — no `Future` type, no `@async`/`@await`
  tokens, no state-machine transform, no `poll` intrinsic.

This document is intentionally brief; the implementation plan for the coroutine
model will be written when that phase begins.
