# Research: Zig's async — the round trip that proves the thesis

Background note for [`zen-concurrency-model.md`](zen-concurrency-model.md). Zig is
the most important data point we have, because Zig **tried the keyword/stackless
design, removed it, and came back with the allocator-style parameter + stackful
fibers** — independently arriving at Zen's model.

---

## Act 1 (Zig 0.5–0.10): stackless coroutines + keywords + the one great idea

Zig's async was **stackless coroutines**: an `async` function was transformed by the
compiler into a **state machine**, with `suspend`/`resume` as the suspension points
and `await` to get the result.

The genuinely great idea — and the part that bears directly on Zen — was that the
**coroutine frame was a first-class, sized value you place yourself**:

- `@Frame(func)` — the concrete frame *type* for a function.
- `@frameSize(func)` — its size, known at comptime.
- `@asyncCall(frame_buffer, ...)` — invoke an async function **into a buffer you
  provide**.
- `anyframe` / `@frame()` — a generic handle to the running frame.

```zig
var frame: @Frame(add) = async add(1, 2);
const result = await frame;
```

Because the frame is an explicit sized object, **you decide where it lives** — on the
stack, in a pool, or from an allocator. No hidden heap allocation. This "the async
frame is allocator-placed, not magic" instinct is exactly the spirit Zen wants — and
note the frame had to be "large enough to accommodate the longest possible call
chain," which is the stackless analogue of a coroutine stack.

It was also **partly colorblind**: `async` was a property of the *call*, not the
definition. The compiler analyzed whether a function could suspend; the same source
could often be driven both ways. Closer to no-color than JS/Rust — but you still
wrote `suspend`/`await`, and the model was fundamentally keyworded.

## Act 2 (Zig 0.11): the deletion

In 0.11 the `async`/`await` keywords **stopped compiling** and `std.event.Loop` was
deleted. The stated reasons are the lesson:

- **Cost:** Andrew Kelley estimated async was **~1/3 of the compiler's complexity**
  while serving **~5% of use cases.**
- **Coupling:** the design was welded to stackless coroutines and couldn't serve the
  full range — single-threaded microcontrollers up to millions of event-driven
  connections — from one model.

This is the strongest possible warning against putting the async transform *in the
compiler*. Zen's earlier `@async`/`@await` milestone was walking straight into the
same 1/3-complexity trap; we are deleting it for the same reason Zig did.

## Act 3 (Zig 0.16, 2026): the comeback — and it's our model

Zig returned with a design that **sidesteps function coloring entirely**:

> "The `Io` type works analogously to Zig's existing `Allocator` interface: it is
> passed as a parameter rather than being a global or keyword."

> "The same function works with different execution models … No recompilation of the
> library is required. No code change is required."

> "Both are built on userspace stack switching — sometimes called fibers or stackful
> coroutines — enabling the same function to suspend and resume across event-driven
> callbacks **without any `async` keyword in the function signature**."

So Zig's endpoint is:

1. **Pass the capability (`Io`) as a parameter** — like the allocator. Colorblind.
2. **Stackful coroutines (fibers)** underneath — backends include io_uring (Linux)
   and Grand Central Dispatch (macOS).
3. **No keyword in the signature.** `io.async(...)` / `io.asyncConcurrent(...)` are
   *methods on the passed capability*, not language keywords coloring the function.

## What Zen takes from the whole journey

| Zig phase | Lesson | Zen |
|---|---|---|
| `@Frame`/`@frameSize` (Act 1) | The coroutine frame should be an explicit, **allocator-placed** object, not compiler magic. | The coroutine **stack** is allocated by the allocator. Zig's "you place the frame" becomes "the allocator owns the fiber stack." |
| The 1/3-complexity deletion (Act 2) | Putting the async **transform in the compiler** is ruinously expensive and inflexible. | Zen puts **zero** async in the compiler — stackful via libc `ucontext`, all stdlib. |
| `Io`-as-parameter + fibers (Act 3) | Pass the capability as a parameter; stackful underneath; no keyword → no color. | **Exactly Zen's model**, with the capability folded into the **allocator** (memory strategy × Sync/Async mode), dispatched by static monomorphization. |

> Zig needed three acts and several years to get from "stackless + keywords + a third
> of the compiler" to "pass a capability + fibers + no color." Zen starts at Act 3,
> folds the capability into the allocator, and keeps the compiler at zero async cost.

One open design choice Zig answers differently: Zig keeps **`Io` separate from
`Allocator`** (two capabilities). Zen's instinct is to **fold execution mode into the
allocator** so there's a single thing to pass (`Alloc(Async, Arena)`). Both are
defensible; see [`zen-async-implementation.md`](zen-async-implementation.md) for how
Zen draws the line (an executor the allocator carries, so one parameter serves both).

---

### Sources
- [zig.guide — Async frames & suspend blocks](https://zig.guide/async/frames-suspend/), [Async/Await](https://zig.guide/master/async/async-await/)
- [ziglang/zig #2377 — The Coroutine Rewrite Issue](https://github.com/ziglang/zig/issues/2377); [#6025 async/await/suspend/resume](https://github.com/ziglang/zig/issues/6025); [#23446 stackless coroutines as low-level primitives](https://github.com/ziglang/zig/issues/23446)
- [Ziggit — What is the status of async with Zig?](https://ziggit.dev/t/what-is-the-status-of-async-with-zig/5715)
- [Machine Herald — Zig 0.16 Nears Release with a Reinvented Async I/O That Sidesteps Function Coloring (2026)](https://machineherald.io/article/2026-03/10-zig-016-nears-release-with-a-reinvented-async-io-that-sidesteps-function-coloring/)
