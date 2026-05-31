# Zen async, implemented cleanly

How the model in [`zen-concurrency-model.md`](zen-concurrency-model.md) is actually
built — grounded in [Zig 0.16's endpoint](zig-async.md) (pass the capability as a
parameter, stackful fibers, no keyword) but with the capability folded into the
allocator and **zero async machinery in the compiler**.

The design is five thin layers. Code is illustrative Zen (final names may differ).

---

## Layer 0 — the one irreducible primitive: a stack switch (libc, via FFI)

Saving the CPU stack and resuming another is the *only* thing that can't be written
in portable Zen. libc already provides it — so it's an `@extern`, **not** a `@builtin`,
and the compiler stays at zero async cost.

```zen
// stdlib/concurrency/coroutine.zen
@extern getcontext  = (ctx: RawPtr<u8>) i32
@extern makecontext = (ctx: RawPtr<u8>, entry: RawPtr<u8>, argc: i32) void
@extern swapcontext = (save_to: RawPtr<u8>, jump_to: RawPtr<u8>) i32
// ucontext_t is ~936 bytes on x86-64 Linux; we just hand the kernel a buffer.
```

## Layer 1 — the coroutine: an allocator-placed stack + saved context

This is Zig's `@Frame` insight made stackful: **the fiber's stack is allocated by an
allocator** (you decide where it lives and how big), not by compiler magic.

```zen
Coroutine: {
    ctx:      RawPtr<u8>,   // this fiber's ucontext_t
    ret_ctx:  RawPtr<u8>,   // where to jump back to (the resumer)
    stack:    RawPtr<u8>,   // the fiber's stack — ALLOCATOR-PLACED
    finished: bool,
}

// `a` provides the stack; `entry` is a Zen fn (we've proven fns cross @extern, see
// the pthread sync primitives). The entry reads "current coroutine" from a
// scheduler global because makecontext can't pass rich args.
coroutine_new<A: Allocator> = (a: A, entry: RawPtr<u8>, stack_size: i64) Coroutine {
    stack = a.alloc(cast(stack_size, usize))
    ctx   = a.alloc(cast(936, usize))
    getcontext(ctx)
    // ... wire ctx.uc_stack = {stack, stack_size}, makecontext(ctx, entry, 0) ...
    Coroutine { ctx: ctx, ret_ctx: a.alloc(cast(936, usize)), stack: stack, finished: false }
}

coroutine_resume = (c: Coroutine) void { swapcontext(c.ret_ctx, c.ctx) }  // into the fiber
coroutine_yield  = (c: Coroutine) void { swapcontext(c.ctx, c.ret_ctx) }  // back to resumer
```

`swapcontext` preserves the *entire* call stack automatically — that is why an
arbitrarily deep callee can suspend with **no `await` on the way up, no color**.

## Layer 2 — the capability: an `Executor` (Sync vs Async)

The thing that actually decides "block vs yield." A blocking op on a `Sync` executor
calls the syscall directly; on an `Async` executor it registers with the reactor
(epoll/io_uring — both already real Zen) and **yields the current fiber**, to be
resumed when the fd is ready.

```zen
Executor: behavior {
    read:  (Self, fd: i64, buf: RawPtr<u8>, len: i64) i64
    write: (Self, fd: i64, buf: RawPtr<u8>, len: i64) i64
}

SyncExec: { }
SyncExec.implements(Executor) {
    read  = (self: SyncExec, fd: i64, buf: RawPtr<u8>, len: i64) i64 { sys3(0, fd, ptr(buf), len) } // blocks
    write = (self: SyncExec, fd: i64, buf: RawPtr<u8>, len: i64) i64 { sys3(1, fd, ptr(buf), len) }
}

AsyncExec: { reactor: RawPtr<u8> }
AsyncExec.implements(Executor) {
    read = (self: AsyncExec, fd: i64, buf: RawPtr<u8>, len: i64) i64 {
        reactor_wait_readable(self.reactor, fd)   // register fd, coroutine_yield until ready
        sys3(0, fd, ptr(buf), len)                // now it won't block
    }
    write = (self: AsyncExec, fd: i64, buf: RawPtr<u8>, len: i64) i64 { ... }
}
```

## Layer 3 — fold it into the allocator: `Alloc(mode, strategy)`

Zig keeps `Io` and `Allocator` separate; Zen's instinct is **one capability to pass**.
An allocator carries its executor, so `Alloc(Async, Arena)` gives you arena memory
*and* yielding I/O from a single value. (You can still split them if you ever want
to; bundling is the ergonomic default.)

```zen
//                strategy axis →   Arena       Heap       Pool
//  mode ↓
//  Sync   (exec blocks)          SyncArena   SyncHeap   SyncPool
//  Async  (exec yields fiber)    AsyncArena  AsyncHeap  AsyncPool
//
// All implement Allocator (alloc/realloc/free) AND forward Executor (read/write/…).
```

## Layer 4 — the payoff: one multisync function, no color, zero cost

Written **once**, generic over the allocator. No `async`, no `await`, no `Future`,
nothing in the signature that says "I might suspend":

```zen
copy_all<A: Allocator> = (a: A, src: i64, dst: i64) i64 {
    buf = a.alloc(cast(4096, usize))
    total ::= cast(0, i64)
    loop((l) {
        n = a.read(src, buf, 4096)          // the only suspend point — and it's invisible
        more = n > 0
        more ?
            | true {
                a.write(dst, buf, n)
                total = total + n
                l.next()
            }
            | false { l.done() }
    })
    a.free(buf, cast(4096, usize))
    total
}
```

- `copy_all<SyncArena>(sync_arena(), …)` → Zen monomorphizes `a.read` to the blocking
  impl → a straight-line synchronous program. The fiber machinery is never compiled in.
- `copy_all<AsyncArena>(async_arena(), …)` → the *same source* monomorphizes `a.read`
  to the yielding impl → a cooperative fiber.

**"Matching on the allocator" is the monomorphization** — Zen's behavior bounds are
100% static (no vtable, no dynamic dispatch), so the choice costs **zero runtime
instructions** and the wrong path is never even emitted for a given instantiation.
This is strictly better than Zig (no keyword at all) and better than Nim (no `Future`
in the type, callable from anywhere).

## Layer 5 — scheduler + actors on top (mostly already built)

```zen
Scheduler: { ready: Vec<Coroutine>, reactor: RawPtr<u8> }
// run(): resume each ready fiber; when one yields on I/O it parks in the reactor;
// poll the reactor (epoll/io_uring) for readiness and move fibers back to `ready`.
```

Actors (`concurrency/actor/{actor,system,supervisor}`, already real) become: an
isolated state + a mailbox `channel` + a fiber draining it. Hand an actor an `Async`
allocator and it's a green-threaded, cooperatively-scheduled actor (Go/Hollywood
ergonomics, Pony isolation); hand it a `Sync` allocator and the identical code is a
blocking worker. **The actor's allocator is its execution mode.**

---

## Why stackful (and the honest cost)

| | Stackless (Zig v1) | **Stackful (Zen)** |
|---|---|---|
| Compiler cost | state-machine transform — *~1/3 of Zig's compiler* | **zero** (libc `ucontext`) |
| Keywords / color | needs `suspend`/`await` markers | **none** — suspension hides in the executor method |
| Frame memory | exact size, cheap | a whole stack per fiber (fat) |
| Suspend anywhere | only at marked points | **anywhere** — deep callees suspend freely |

Zig deleted stackless precisely because the transform was too costly and inflexible.
Zen refuses to put *any* of that in the compiler, so we take stackful and pay in
**stack memory** instead. The cost is real but **the allocator is the mitigation**:
the fiber stack is allocator-placed (Layer 1), so an `Arena` packs many small fiber
stacks contiguously, a `Pool` reuses fixed-size stacks across fibers, and the caller
sizes the stack for the workload. The thing that makes async cheap is the same thing
that makes async *exist*: the allocator.

---

## Build order

1. `coroutine.zen` over libc `ucontext` — spawn/resume/yield; fixture: a fiber yields
   N times and resumes to completion (deterministic count).
2. `scheduler.zen` — run-queue; fixture: 3 fibers round-robin to completion.
3. `Executor` behavior + `SyncExec`/`AsyncExec`; the `AsyncExec` reactor over the
   already-real epoll/io_uring; fixture: one `copy_all` runs identically under
   `SyncArena` and `AsyncArena` (same output), proving multisync.
4. Bundle executor into the `Sync*`/`Async*` allocators (`Alloc(mode, strategy)`).
5. Re-promote the actor framework on fibers; per-actor allocators.

Every layer is stdlib Zen over `@extern` + existing `@builtin` hooks. The compiler's
async surface stays at **zero**.

---

### Related
- [`zen-concurrency-model.md`](zen-concurrency-model.md) — the why and the synthesis
- [`zig-async.md`](zig-async.md) — Zig's round trip to this same endpoint
- [`nim-multisync.md`](nim-multisync.md) · [`pony-actors.md`](pony-actors.md)
