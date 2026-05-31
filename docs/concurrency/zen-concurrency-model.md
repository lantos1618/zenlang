# Zen Concurrency: Sync and Async by Default, No Function Color

> **Thesis.** In Zen, *sync vs async is a property of the allocator you pass, not of
> the function you write.* There is no `async`/`await`, no `Future` keyword, no
> "red vs blue" function coloring. A function is written **once**, generic over its
> allocator; hand it a sync allocator and it blocks, hand it an async allocator and
> it yields a coroutine. The whole model rolls up and out of the **memory/allocator
> system** plus three things Zen already has — **static behavior monomorphization**,
> **stackful coroutines**, and **actors**. The compiler grows by *nothing*; in fact
> it shrinks (the async keywords are deleted).

This document explains the model, why it is better than the languages that inspired
it, and exactly which pieces live where. See the companion research notes:
[`nim-multisync.md`](nim-multisync.md) and [`pony-actors.md`](pony-actors.md).

---

## 1. The problem we are deleting: function color

Bob Nystrom's "[What Color is Your Function?](https://journal.stuffwithstuff.com/2015/02/01/what-color-is-your-function/)"
names the disease of `async`/`await` languages:

- **Red** (async) functions can only be called from other red functions.
- **Blue** (sync) functions can be called from anywhere.
- `await` is the only bridge, and it only works *inside* red.

Red is contagious. One async leaf forces every transitive caller to become async,
all the way up to `main`. You end up maintaining two copies of everything (a sync
`read` and an async `readAsync`), or you `block_on` at boundaries and lose the
concurrency. JavaScript, Rust, C#, Python, Kotlin (partially) all carry this tax.

The root cause is mechanical: **without threads or stackful coroutines, a suspend
point has to "closurify" the entire callstack back to `main`** — so the suspension
has to be visible in every type signature on the way up. That visibility *is* the
color.

Languages that **don't** have the problem — Go, Lua, Ruby fibers — share one trait:
**multiple independent callstacks you can switch between** (stackful coroutines).
"Go does not color its functions at all. Any function has the right to suspend its
goroutine by doing an IO operation." That is the property we want.

Zen had `@async`/`@await` for exactly one milestone. They are being **removed**.

---

## 2. The three ideas we steal

| Source | What it gets right | What Zen takes |
|---|---|---|
| **Nim `multisync`** | One procedure body serves both sync and async; you don't write it twice. | *Write once.* But Nim only **de-duplicates code** — the async copy still returns `Future` and is still red. Zen goes further: **zero color**, not just zero duplication. |
| **Go / Lua / fibers** | Stackful coroutines → any function can suspend → **no color**. | *The suspension mechanism.* Stackful coroutines via libc `ucontext`. (Go's channels are a footgun museum; we take the coroutines, not the channels.) |
| **Pony actors** | Compile-time **data-race-free** concurrency; actors with their own heaps; cheap; cooperative core-bound scheduler. | *The safety model and the actor framework.* Isolated state, message passing, per-actor allocation. |

The synthesis: **Pony's safety + Go's cheap coroutines + Nim's write-once, taken to
zero color — and the switch is the allocator.**

---

## 3. How Zen does it

### 3.1 The one irreducible primitive — a stack switch (and it's not even a builtin)

A stackful coroutine needs to save the current CPU stack and resume another. That
is the *only* thing that cannot be written in portable Zen. And we don't need a
compiler feature for it — libc already exposes it:

```
@extern getcontext  = (ctx: RawPtr<u8>) i32
@extern makecontext = (ctx: RawPtr<u8>, fn: RawPtr<u8>, argc: i32) void
@extern swapcontext = (out: RawPtr<u8>, in: RawPtr<u8>) i32
```

We have already proven a Zen function can be passed across `@extern` as a callback
(that's how the pthread-based `concurrency/sync/*` works). So a coroutine is:
allocate a stack, `makecontext` it onto a Zen entry function, and `swapcontext` to
run/suspend it. **Pure stdlib FFI. The compiler's async surface is literally zero
lines.**

> Net effect of this whole redirection: the compiler gets *thinner*. We delete the
> `@async`/`@await` tokens, the `Future` type, the state-machine lowering pass
> (`async_lowering.rs`), and the `poll` intrinsic, and add nothing in their place.

### 3.2 The switch is the allocator — and "matching on it" is free

Execution mode is one axis of the allocator, orthogonal to strategy:

```
            strategy →     Arena        Heap         Pool
 mode ↓
 Sync                   SyncArena     SyncHeap     SyncPool      ← ops block
 Async                  AsyncArena    AsyncHeap    AsyncPool     ← ops yield the coroutine
```

All of them implement the same `Allocator` behavior. The difference is *what their
operations do when they'd have to wait*:

- A **Sync** allocator's operation **blocks** the thread (a plain syscall).
- An **Async** allocator's operation, when it would block, **`swapcontext`es back to
  the scheduler** — the current coroutine suspends, another runs, and the scheduler
  resumes this one when the resource is ready.

Now the key move. You write your logic **once**, generic over the allocator:

```
do_work<A: Allocator> = (a: A) Result<Data, Error> {
    buf = a.alloc(4096)          // <- the only "await point", and it's invisible
    n   = read_through(a, fd, buf)
    parse(buf, n)
}
```

Call it `do_work<SyncArena>(...)` and Zen monomorphizes `a.alloc` to the blocking
impl — a straight-line synchronous program. Call it `do_work<AsyncArena>(...)` and
the *same source* monomorphizes `a.alloc` to the yielding impl — a coroutine that
suspends and resumes. **There is no runtime `match`, no branch, no vtable.** Zen's
behavior bounds are 100% statically monomorphized (no dynamic dispatch — a core
language decision), so *"matching on the allocator to decide sync vs async" is the
monomorphization itself.* It costs zero instructions.

That is multisync — write once, run either way — but unlike Nim it is **zero color**
(no `Future` in the signature, the function is callable from anywhere) and **zero
cost** (the wrong path is never even compiled into a given instantiation).

### 3.3 Actors on top (the Hollywood ergonomics, the Pony safety)

Actors are already real Zen (`concurrency/actor/{actor,system,supervisor}`). On the
coroutine foundation they become the high-level concurrency model:

- An **actor** = an isolated state + a mailbox (a `channel`) + a coroutine that
  drains the mailbox and reacts to messages. "Don't call us, we'll call you" — the
  Hollywood principle.
- Messages are **fire-and-forget** sends (Pony's *behaviours*); synchronous queries
  are plain function calls (Pony's *functions*).
- Each actor allocates from **its own allocator** — which is exactly where the
  sync/async choice lives. An actor handed an `AsyncArena` is a green-threaded,
  cooperatively-scheduled actor (Go/Hollywood style); the same actor code handed a
  `SyncArena` is a blocking worker. Same code.

Go's `anthdm/hollywood` engine is gorgeous (ring-buffer inboxes, backpressure,
10M msgs/sec) *despite* goroutines+channels being full of footguns. Zen aims for
that ergonomics on a cleaner base: Pony-style isolation for safety, stackful
coroutines for no-color suspension, and the allocator as the sync/async switch.

---

## 4. Where every piece lives

```
┌──────────────────────────── COMPILER (Rust) ────────────────────────────┐
│  Async surface: ZERO.                                                     │
│  Already has, and that's all it needs:                                   │
│    • generics + behaviors with 100% STATIC monomorphization              │
│      (this is the "match on the allocator", for free)                    │
│    • @extern FFI (to reach libc ucontext)                                 │
│    • raw memory / syscall @builtin hooks                                  │
└──────────────────────────────────────────────────────────────────────────┘
                                   │  builds on
                                   ▼
┌──────────────────────────────── STDLIB (Zen) ───────────────────────────┐
│  coroutine.zen   — stack + ucontext: spawn / yield / resume (FFI)        │
│  scheduler.zen   — run-queue of coroutines, cooperative swapcontext loop │
│  memory/*        — Allocator behavior; Sync* and Async* impls (the switch)│
│  concurrency/actor/* — actors/system/supervisor over mailbox + coroutine │
│  everything else (io, sync primitives, collections) — already real Zen    │
└──────────────────────────────────────────────────────────────────────────┘
```

A program then just chooses:

```
main = () i32 {
    // one line decides the whole program's execution model
    a ::= async_arena(1 << 20)        // or sync_arena(...) — same code below
    sys ::= actor_system(a)
    sys.spawn(worker)                 // worker is multisync; it follows `a`
    sys.run()
    0
}
```

---

## 5. Why this is strictly better than each inspiration

- **vs Nim multisync** — Nim de-duplicates the *source* but the async path is still
  red (`Future[T]`, `await`). Zen has **no color at all**: `do_work` has no async in
  its type, composes with any caller, and the sync instantiation has zero coroutine
  overhead.
- **vs Go** — Go gets no-color right via goroutines, but its concurrency *plumbing*
  (channels, `select`, nil/closed-channel panics, goroutine leaks, no structured
  concurrency, no backpressure by default) is error-prone. Zen keeps the coroutines,
  drops the channels-as-primary-API in favor of **actors with typed mailboxes and
  backpressure**, and adds Pony-style isolation.
- **vs Pony** — Pony's safety is world-class but its reference-capability system
  (`iso/val/ref/box/tag/trn`) is a steep wall, and async is *mandatory* (everything
  is an actor). Zen makes async **opt-in by allocator**: the same function is sync or
  async by which allocator you pass, so you pay for concurrency only where you ask
  for it, and the safety story is the allocator + isolated-actor-state model rather
  than a capability lattice.

---

## 6. Status & next steps

- **Phase 1 (in progress):** delete the `@async`/`@await`/`Future`/transform/`poll`
  apparatus. Compiler async surface → 0. The 6 keyword-async stdlib modules revert
  to placeholders, to be re-promoted on this model.
- **Phase 2:** `coroutine.zen` over libc `ucontext` (spawn/yield/resume) — pure
  stdlib FFI; a deterministic fixture (a coroutine that yields N times and resumes).
- **Phase 3:** `scheduler.zen` — cooperative run-queue over the coroutine primitive.
- **Phase 4:** `Sync*`/`Async*` allocator impls — the execution-mode axis; an
  `Async` allocator yields on would-block. Prove a single multisync function runs
  both blocking and cooperatively by changing only the allocator it's given.
- **Phase 5:** rebuild the actor framework on coroutines; per-actor allocators.

The end state: **no function color, write-once multisync, Pony-grade isolation,
Hollywood-grade actor ergonomics — all rolled out of the allocator/memory model,
with a compiler that stays thin.**

---

### Sources
- Bob Nystrom, [*What Color is Your Function?*](https://journal.stuffwithstuff.com/2015/02/01/what-color-is-your-function/)
- Roman Elizarov, [*How do you color your functions?*](https://elizarov.medium.com/how-do-you-color-your-functions-a6bb423d936d)
- [Nim `std/asyncmacro` & multisync](https://nim-lang.org/docs/asyncmacro.html)
- [Pony tutorial — ORCA garbage collection](https://tutorial.ponylang.io/appendices/garbage-collection.html); [Orca: GC and Type System Co-Design (OOPSLA'17)](http://janvitek.org/pubs/oopsla17a.pdf)
- [anthdm/hollywood — actor engine for Go](https://github.com/anthdm/hollywood)
