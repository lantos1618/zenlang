# Research: Nim's `multisync` — one body, sync *and* async

Background note for [`zen-concurrency-model.md`](zen-concurrency-model.md). What Nim
does, how it works, where it stops short, and what Zen takes from it.

---

## What it is

In Nim you normally write asynchronous procedures with the `{.async.}` pragma and
`await`:

```nim
proc fetch(s: AsyncSocket, n: int): Future[string] {.async.} =
  let data = await s.recv(n)
  return data
```

The pain: if you also need a **synchronous** version (blocking sockets, no event
loop), you have to write the whole proc again with blocking calls and no `await`.
Two copies of the same logic that drift apart.

`{.multisync.}` solves the duplication. It is a **macro** that takes a single
procedure definition and **generates both** a synchronous and an asynchronous
version:

```nim
proc readMany(this: Redis | AsyncRedis, count: int = 1): Future[string] {.multisync.} =
  if count == 0:
    return ""
  let data = await this.receiveManaged(count)
  return data
```

- The **async** version is produced by running the normal `async` macro (keeps
  `Future` + `await`, drives the event loop).
- The **sync** version is produced by **stripping the `await`s and the `Future`
  wrapping** — `await x` becomes just `x`, so the proc blocks and returns the plain
  value.

The `this: Redis | AsyncRedis` union type is how one body type-checks against both a
blocking and a non-blocking transport.

## How it works mechanically

`multisync` is pure macro/AST surgery at compile time (`std/asyncmacro`):

1. Parse the proc body.
2. Emit copy A: hand the body to the `async` transform → a stackless state machine /
   closure-iterator returning `Future[T]`, with `await` as the suspension points.
3. Emit copy B: walk the same body, **delete `await`** (replace `await e` with `e`)
   and unwrap `Future[T]` to `T` → an ordinary blocking proc.

So the *programmer* writes once; the *compiler* still produces two differently-typed
functions.

## The benefit

- **No duplicated logic.** Libraries (e.g. the Redis client, HTTP clients) ship one
  implementation that serves both blocking and event-loop users.
- The sync path has **no async overhead** — it's genuinely just the blocking code.

## Where it stops short (why Zen goes further)

`multisync` removes *duplication*, not *color*:

- The async copy is **still red.** It returns `Future[T]` and is only callable from
  other async code / via the dispatcher. The function-color problem is intact for
  anyone using the async version.
- It relies on Nim's **stackless** async (CPS via closure iterators), so suspension
  points must be syntactically visible as `await`. You cannot suspend in the middle
  of an arbitrary callee without that callee also being async.
- The two versions are selected by **overload/type union** at the call site, and the
  async one infects its callers exactly as normal `async` does.

In short: Nim multisync is "write the body once, but you still have a red function
and a blue function." It is a code-organization win, not a coloring fix.

## What Zen takes

- **The write-once goal** — one source serves sync and async.
- **And then removes the catch.** Because Zen uses **stackful coroutines** (a real
  stack switch, not a `Future` state machine) and decides sync-vs-async by the
  **allocator type** via static monomorphization, the single Zen function has **no
  `Future` in its signature and no color at all** — it is callable from anywhere, and
  the sync instantiation has zero coroutine cost. The suspension points are the
  allocator's operations, not syntactic `await`s, so an arbitrary deep callee can
  suspend without anyone above it changing color.

> Nim: one body → a red function and a blue function.
> Zen: one body → **one colorless function**, specialized by the allocator you pass.

---

### Sources
- [Nim `std/asyncmacro`](https://nim-lang.org/docs/asyncmacro.html), [`std/asyncdispatch`](https://nim-lang.org/docs/asyncdispatch.html)
- [Nim Days — Redis client (multisync in practice)](https://xmonader.github.io/nimdays/day13_redisclient.html)
- [Peter's DevLog — Asynchronous programming in Nim](https://peterme.net/asynchronous-programming-in-nim.html)
