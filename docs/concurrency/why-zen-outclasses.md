# How Zen Outclasses the Field

A fair, concrete comparison on the axis Zen was built for: concurrency, function
color, and the thin-compiler bet. Every Zen claim here is backed by a **green
runtime fixture on master**, not a roadmap. See [`zen-concurrency-model.md`](zen-concurrency-model.md)
for the design and [`zen-async-implementation.md`](zen-async-implementation.md) for the code.

---

## The one table

| Language | Function **color**? | Async machinery **in the compiler** | Who controls the **frame/stack** | Suspend mechanism | GC pauses |
|---|---|---|---|---|---|
| **Rust** | **Yes** — `async fn` infects callers | Large: `Future`/`Pin`/`Unpin`/`Waker`/`poll`, the transform | runtime/executor | stackless state machine | none |
| **JS / C# / Python** | **Yes** — `async`/`await` everywhere | in runtime | runtime | event loop / stackless | varies |
| **Nim** | **Yes** — `multisync` de-dups source, async copy still returns `Future` | macro (CPS closures) | runtime | stackless | yes |
| **Go** | **No** | scheduler baked into the runtime | hidden (growable stacks) | stackful goroutines | yes (low-pause) |
| **Pony** | **No** (but *everything* is an actor) | runtime + the type system | per-actor heap | actor scheduler | per-actor (ORCA), concurrent |
| **Zig 0.16** | **No** | minimal — `Io` interface passed in | explicit, but `Io` is **separate** from `Allocator` | stackful fibers | none |
| **Zen** | **No** | **ZERO lines** | **the allocator places the fiber stack** | stackful fibers (libc `ucontext`, **stdlib**) | none |

Two columns decide it: **color** and **async-in-the-compiler**. Zen is the only row
that is *No* and *Zero* at once — and folds the capability into the allocator you
were already passing.

---

## The proof, not the promise

This compiles, links, and runs on master today (`tests/zen/stdlib_multisync.zen`):

```zen
// Written ONCE. No @async, no @await, no Future. Nothing in the type says "suspends".
pump<E: Executor> = (e: E, cell: RawPtr<u8>, n: i64) void {
    i ::= cast(0, i64)
    loop((l) {
        i >= n ? | true { l.done() } | false { e.step(cell)  i = i + 1  l.next() } })
}

pump<SyncExec>(SyncExec { }, sc, 5)    // → runs straight through        →  sync=5
// the SAME source, under an Async executor, inside a coroutine:
pump<AsyncExec>(AsyncExec { }, g_cell, 5)  // → suspends/resumes each step →  async=5, yields=5
```

Output: `sync=5 / async=5 / async_yields=5`. One function. Two execution models. **No
color, no `Future`, no runtime branch** — the choice is static behavior
monomorphization (the wrong path is never even emitted).

---

## Head to head

### vs Rust — the color tax, gone
Rust's `async fn` returns an opaque `Future`; only `async` callers can `await` it, so
red infects the whole call graph up to an executor. You also pay the `Pin`/`Unpin`/
`Waker` mental tax and maintain sync+async twins of libraries. **Zen has no color**:
`pump` above is callable from anywhere, and the sync instantiation has *zero* coroutine
overhead. Rust's async is a large, permanent part of the language; Zen's is **0 lines
of compiler**.

### vs Go — the ergonomics, without the footguns or the GC
Go also has no color (goroutines suspend on I/O) — its one great idea, which Zen keeps.
But Go bakes the scheduler into the runtime, hides allocation, ships a GC (pauses,
however small), and leans on **channels** whose nil/closed/`select` semantics are a
notorious footgun museum with no backpressure by default and easy goroutine leaks.
**Zen**: the scheduler is readable stdlib Zen; the fiber stack is **allocator-placed**
(you choose where memory lives); **no GC**; and the concurrency API is **typed actor
mailboxes with backpressure**, not raw channels.

### vs Zig — same destination, no detour, and one capability instead of two
Zig is the closest, and its journey is the proof Zen is right: stackless + keywords →
**deleted** (async was ~⅓ of the compiler for ~5% of use cases) → **Zig 0.16: "`Io`
passed as a parameter like the Allocator interface… fibers… no `async` keyword."**
Zen *starts* at that endpoint. Difference: Zig keeps **`Io` and `Allocator` as two
capabilities**; Zen **folds execution mode into the allocator** — `Alloc(Async, Arena)`
— so there's one thing to thread, and it carries memory strategy *and* sync/async mode
together.

### vs Nim — write-once, and actually colorless
Nim's `multisync` is the right instinct (one body, sync + async), but it only removes
*duplication*: the generated async proc still returns `Future[T]` and is still red. **Zen's
single function has no `Future` in its type at all** and the sync build has no async
cost. Write-once *and* color-free, not just write-once.

### vs Pony — the safety ethos, without mandatory async or the capability wall
Pony is the gold standard for compile-time data-race freedom — but you buy the entire
reference-capability lattice (`iso`/`val`/`ref`/`box`/`tag`/`trn`) and **everything is an
actor** (async is mandatory). **Zen** takes Pony's actor isolation and per-actor
allocation (the load-bearing idea) and makes async **opt-in by allocator**: the same
function is sync or async by which allocator it gets, so you pay for concurrency only
where you ask, and there's no capability lattice to climb first.

---

## The deeper bet: the compiler stays thin

The concurrency win is a symptom of the real thesis — **the compiler owns only the
language and raw hooks; the entire runtime is real, tested Zen.**

- **Compiler:** ~20,313 lines, ~65 `@builtin` hooks, **0 TODOs**, **0 lines of async**.
  We *deleted* 717 lines when we moved async to stdlib and it got *better*.
- **Runtime, in Zen you can read:** allocators (`Arena`/`Pool`/`Heap`/composite `GPA`),
  syscalls + io (files, sockets, epoll, io_uring constants, timers, signals), real
  pthread threading and the full sync-primitive suite, actors, **coroutines, scheduler,
  tasks, and the Sync/Async executor** — **56 of 63 stdlib modules promoted, every one
  with a runtime fixture.**

Most languages bury their runtime in compiler magic you can't see or change. Zen's is a
library you can read, audit, and replace — and the async model proves the bet: the
hardest runtime feature in modern languages needed **zero** compiler support here.

> Rust pays the color tax. Go pays the GC + footgun tax. Nim only de-dups. Pony makes
> you climb the capability wall and go all-actor. Zig took years and a painful deletion
> to reach "pass the capability + fibers." **Zen is colorless, GC-free, allocator-driven,
> proven green, and its compiler doesn't know what async is.**

---

### Related
[`zen-concurrency-model.md`](zen-concurrency-model.md) · [`zen-async-implementation.md`](zen-async-implementation.md) · [`zig-async.md`](zig-async.md) · [`nim-multisync.md`](nim-multisync.md) · [`pony-actors.md`](pony-actors.md)
