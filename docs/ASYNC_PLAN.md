# Async program — scope & sequencing

Status: **not started** (only stubs exist). This is the one track where "the
compiler is fine, work in stdlib" does **not** hold — async needs an irreducible
compiler capability that stdlib cannot fake.

## What exists today (the honest baseline)

- Two void intrinsic stubs: `async_enqueue(task)` and `async_yield()`
  (`src/intrinsics/definitions.rs:124-125`). They type-check and lower to no-ops.
- Every async stdlib file is a parse-only placeholder:
  `stdlib/concurrency/async/{task,scheduler}.zen`,
  `stdlib/concurrency/actor/async_actor.zen`,
  `stdlib/memory/async_{helpers,pool}.zen`, `stdlib/io/mux/uring.zen`.

There is **no suspension mechanism**: nothing can pause a function mid-body and
resume it later. That is the whole problem.

## The one irreducible compiler piece

An `async` function must be able to suspend at an await point and resume with all
locals intact. In a C backend that means the compiler must transform an async
function into a **resumable state machine**:

- split the function body at each await point into states,
- spill live locals into a heap-allocated frame (the coroutine frame),
- generate a `resume(frame)` entry that switches on the saved state,
- return a `Poll`-like value (Pending / Ready(T)) to the caller.

This is a real codegen feature (CPS / state-machine lowering). It cannot be
expressed in stdlib Zen on the current `@builtin` hooks — it needs the compiler
to restructure control flow and manage frames. Everything else (scheduler, I/O
mux, async allocator) is stdlib on top of it.

## Milestone ladder (each independently shippable, in order)

1. **Async/await MVP — single-threaded, no I/O.**
   - `async` function marker + `await` expression in parser/typechecker.
   - State-machine lowering for one await point, then N await points, then
     awaits inside loops/branches.
   - Frame allocation via the existing `Allocator` (so it composes with the
     sync allocators we just built — async frames can live in an Arena/Pool).
   - Proof: an async function that awaits a ready value and returns it; then one
     that awaits twice and threads a local across the suspend.

2. **Cooperative scheduler.** Promote `stdlib/concurrency/async/scheduler.zen`:
   a run-queue of pending frames, `spawn`, `block_on`. `async_enqueue`/
   `async_yield` get real lowering or are replaced by scheduler calls.
   Proof: spawn three tasks that round-robin via yield and finish in order.

3. **I/O readiness (the point of async).** `stdlib/io/mux` over epoll/io_uring:
   register interest, park the frame, resume on readiness. Needs the mux
   syscalls as intrinsics or extern FFI.
   Proof: two sockets/timers awaited concurrently complete out of submission
   order.

4. **Async allocator — LAST.** Only here does the Sync/Async *execution-mode*
   axis become real (`Arena<Async>` etc.): an allocator whose `alloc` can await
   (e.g. backpressure when a pool is exhausted). Until milestone 1 exists, an
   async allocator has nothing to await on, which is why the phantom `<E>`
   execution-mode type param was deliberately NOT added to the sync allocators.

## Why allocator-async is sequenced last

The two-axis vision is {Sync, Async} × {Arena, Heap, Pool}. The **strategy** axis
(Arena/Heap/Pool) is done today, purely in stdlib. The **execution-mode** axis is
meaningless without a real await: a "Sync" allocator is just the allocators we
have, and an "Async" allocator is milestone 4. Adding the marker types now would
be speculative scaffolding with no runtime behind it.

## Rough size

Milestone 1 alone is a multi-week compiler effort (new lowering pass, frame ABI,
typechecker support for `async`/`await`). Milestones 2-4 are each comparable.
This is a program, not a task — treat each milestone as its own goal.
