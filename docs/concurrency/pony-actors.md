# Research: Pony's actor model — data-race-free concurrency, by construction

Background note for [`zen-concurrency-model.md`](zen-concurrency-model.md). What
Pony does, why it's safe *at compile time*, and what Zen takes (and deliberately
leaves).

---

## The shape of Pony

Pony is a statically-typed, **actor-model** language. A program is a swarm of
**actors** that own isolated state and communicate only by **asynchronous message
passing**. There are no locks, and there cannot be a data race — the *type system*
proves it before the program runs.

An actor has two kinds of methods:

- **behaviours** (`be name(...)`) — **asynchronous**. Calling one **sends a message**
  to the actor's queue and returns immediately (fire-and-forget; no return value).
  This is the "don't call us, we'll call you" / Hollywood shape.
- **functions** (`fun name(...)`) — **synchronous**, ordinary method calls that
  return a value.

Each actor processes its mailbox **one message at a time**, so its own state is
never touched concurrently — an actor is a sequential island in a concurrent sea.

## The safety mechanism: reference capabilities

Pony's signature idea. Every reference's type carries a **reference capability** that
says how the data may be aliased and shared:

| cap | meaning (roughly) |
|---|---|
| `iso` | **isolated** — the *only* readable+writable alias; safe to send between actors |
| `trn` | **transition** — writable by me, others may have read-only `box` aliases |
| `ref` | mutable, **not** sendable (actor-local) |
| `val` | **immutable + globally shared** — safe to send (deeply immutable) |
| `box` | read-only view (could be `val` or someone else's `ref`) |
| `tag` | **identity only** — no read/write; an actor reference is a `tag` |

The compiler enforces that anything crossing an actor boundary is either **`iso`**
(uniquely owned, so the sender gives it up) or **`val`** (deeply immutable, so
sharing is safe). Result: **"the Pony compiler verifies at compile time that your
code is data-race- and deadlock-free."** No runtime checks, no locks.

## ORCA: garbage collection without stop-the-world

Each actor has its **own heap** and collects it **independently and concurrently**:

- An actor may GC **while other actors run any behaviour**.
- It decides whether to collect an object **from its own local state alone** — no
  consultation with other actors.
- The only cross-actor coordination is **message sends** (deferred, distributed,
  weighted reference counting on objects shared between actors).

So there is **no global GC pause**; collection is per-actor and concurrent. This is
the Orca protocol (a GC/type-system co-design — the capabilities are what make
local, coordination-free collection sound).

## Scheduling

Pony runs actors on a **fixed pool of scheduler threads**, typically **one bound per
core**. Actors are cheap (you can have millions); the scheduler multiplexes runnable
actors onto the threads (work-stealing, M:N). Messaging is **causal** (ordering
guarantees that make reasoning tractable). An actor that isn't processing a message
costs almost nothing.

## What's great — and what's heavy

Great:
- **Compile-time data-race freedom** with **no locks** and **no GC pauses**.
- A clean async (behaviours) / sync (functions) split with the Hollywood ergonomics.
- Actors as the *unit of concurrency, isolation, and collection* all at once.

Heavy:
- **Reference capabilities are a steep learning wall.** `iso`/`trn`/`val`/`box`/`tag`
  and the aliasing/consume rules are powerful but hard.
- **Async is mandatory** — *everything* is an actor; there is no "just call this
  function synchronously without the actor machinery." You opt into the whole model.

## What Zen takes (and leaves)

**Takes:**
- The **actor as isolated state + mailbox + sequential message processing** — already
  real Zen (`concurrency/actor/{actor,system,supervisor}`).
- The **async = message send / sync = function call** distinction (Hollywood shape).
- The **per-actor allocation** idea — and Zen makes it the *load-bearing* idea: an
  actor allocates from its own allocator, and **that allocator's sync/async mode is
  the actor's execution mode.**
- The **compile-time-safety ethos** — push correctness into the type system, not the
  runtime.

**Leaves (for now):**
- The full **reference-capability lattice.** Zen's isolation story is "an actor owns
  its allocator/heap and messages carry values," which is simpler though less
  expressive than Pony's `iso`/`val` proofs. (A capability-lite story may come later.)
- **Mandatory async.** In Zen, async is **opt-in by allocator** — the same function
  is sync or async depending on which allocator you hand it (see the multisync model).
  You pay for concurrency only where you ask for it.

> Pony: safe-by-construction actors, but you buy the whole capability system and
> everything is async.
> Zen: the actor ergonomics and isolation, with async chosen **per allocator** so
> sync code stays sync and colorless.

---

### Sources
- [Pony tutorial — ORCA garbage collection](https://tutorial.ponylang.io/appendices/garbage-collection.html)
- [Orca: GC and Type System Co-Design for Actor Languages (OOPSLA'17, PDF)](http://janvitek.org/pubs/oopsla17a.pdf)
- [The Morning Paper — Ownership and reference counting based GC in the actor world](https://blog.acolyer.org/2016/02/18/ownership-and-reference-counting-based-garbage-collection-in-the-actor-world/)
- [InfoQ — Sylvan Clebsch on Pony's design, GC, and formal verification](https://www.infoq.com/podcasts/sylvan-clebsch-pony-formal-verification/)
- [Introduction to the Pony programming language (opensource.com)](https://opensource.com/article/18/5/pony)
