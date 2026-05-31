# Async program — scope & sequencing

> ## Elevation refactor (current) — compiler surface stripped to the minimum
>
> The async impl below (milestones 1–2) has been refactored so the **compiler
> owns only two things**, and everything else is stdlib Zen:
>
> 1. **The transform** — `@async` body → heap frame struct + `static bool
>    f__poll(void*, void*)` + an allocating constructor; `@await e` →
>    suspend/resume state machine. This is irreducible (`async_lowering.rs`).
> 2. **One thin hook** — `@builtin.poll(frame: RawPtr<u8>, out: RawPtr<u8>) -> bool`,
>    lowering to `(*(zen_poll_fn*)frame)(frame, out)` (true = Ready and wrote
>    `out`, false = Pending). The *entire* driver surface stdlib needs.
>
> **The lang-item (how `Future<T>` is nameable everywhere).** `Future` is a
> **built-in generic type name** — exactly like `RawPtr<T>` / `Slice<T>`. The
> parser maps `Future<T>` → `AstType::Future(T)` via `BuiltinGenericTypeName::Future`,
> and `resolve_type` maps that to `Type::Future(T)`. Because it is a builtin name
> (not a module symbol), it resolves **without an import**, so `@async`/`@await`
> work in any program; and because stdlib writing `Future<i64>` resolves through
> the *same* path, it gets the *same* `Type::Future(i64)` the compiler produces
> for an `@async` call. A `Future<T>` exposes one field, `frame: RawPtr<u8>` (the
> coroutine-frame pointer the transform allocates); its C value *is* that pointer,
> so `.frame` lowers to identity. This is the cleanest option: zero prelude
> injection, zero special module resolution — a future is a builtin type the same
> way a raw pointer is.
>
> **Elevated to stdlib (pure Zen over `@builtin.poll`):**
> - `stdlib/concurrency/async/future.zen` — `block_on<T>(f: Future<T>) T`:
>   `out = raw_allocate(sizeof<T>()); loop { poll(f.frame, out) ? done : next };
>   load<T>(out)`. The compiler's special-cased C `block_on` driver, its
>   typechecker arm, and its resolver whitelist entry are **deleted**.
> - `stdlib/concurrency/async/scheduler.zen` — the cooperative `Scheduler` is a
>   pure-Zen run-queue (a `Vec` of frame pointers) round-robin polling each
>   pending frame via `@builtin.poll`. The compiler `scheduler_new/spawn/run`
>   primitives and the `zen_scheduler` C runtime are **deleted**.
>
> The sections below are the original milestone log; treat the box above as the
> authoritative description of the *current* compiler surface.

---

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

---

## Milestone 1 design (in progress)

### Surface spelling — and why

Zen is keyword-free: every "magic" word is an `@`-directive (`@std`, `@builtin`,
`@this`, `@export`, `@extern`) or a sigil. Async therefore uses two new
`@`-directives, mirroring the existing ones exactly so it slots into the
established dispatch points rather than inventing a parallel grammar:

- **`@async` — the async-function marker.** It prefixes the *function literal*,
  immediately before the parameter list:

  ```zen
  ready = @async (n: i32) i32 { @await pending(n) }
  ```

  This is the natural home: the existing parser already dispatches on `Token::Assign`
  → `parse_function_def`, and a normal function literal is recognised by a leading
  `(`. `@async` is consumed right before that `(`, parallel to how `@extern`
  prefixes an extern function. It sets `is_async: true` on `Declaration::Function`
  (the field defaults to `false`, so every existing AST/golden is byte-identical).

- **`@await e` — the await expression.** A prefix `@`-directive in expression
  position, parallel to `@this`. It parses its operand at prefix binding power
  (so `@await f(x)` awaits the *call* `f(x)`, and `@await a + b` is `(@await a) + b`,
  matching how `-a + b` already associates). Chosen over a postfix `.await` because
  postfix would collide with the existing member-access (`Token::Dot`) parse and
  read as a field named `await`.

Both are gated behind their directive tokens, so a program that uses neither is
lexed, parsed, typed, and emitted exactly as before.

### Typing model (MVP)

- A new typed type `Type::Future(Box<Type>)`. An `@async` fn whose declared
  return type is `T` has its *logical* return type `T` but its **callable**
  return type recorded as `Future<T>`: calling an async function yields
  `Future<T>`, never `T` directly.
- `@await e` requires `e : Future<T>` and produces `T`. It is **only** legal
  inside an `@async` function body (tracked by a `current_fn_is_async` flag on
  the checker, saved/restored around `check_function`, exactly like
  `current_return_type`).
- The async fn *body* is checked against the logical `T` (so `@await pending(n)`
  whose type is `i32` satisfies an `i32`-returning async fn), while the symbol
  table advertises `Future<T>` to callers.

Diagnostics (stable codes, type category):

- `E3080` — `@await` used outside an `@async` function.
- `E3081` — `@await` applied to a non-future (`e` is not `Future<_>`).

These ship with parser + typechecker unit tests. Lowering for the general case
(N awaits, awaits in loops/branches, live-local spilling) is **not** in this
slice; see the ABI sketch below for the next step.

### Frame ABI (target for the lowering step)

Each async fn `f` lowers to:

- a heap-allocated **frame struct** `f__frame { int __state; /* spilled live
  locals */; T __ret; }`, where `__state` is the resume label (0 = start, k =
  "resumed after k-th await", -1 = done);
- a **poll function** `bool f__poll(f__frame* fr, T* out)` returning `true` when
  `Ready` (and writing `*out`), `false` when `Pending`. The body is a `switch
  (fr->__state)` with `case` labels at each await point; live locals are read
  from / written back to `fr` across each suspend;
- a **constructor** `f__frame* f(args...)` that allocates the frame (via the
  default `Allocator`), stores the args, sets `__state = 0`, and returns it —
  this is what a *call* to `f` produces (a `Future<T>`);
- `@await e` lowers, inside a poll body, to: poll `e`; if `Pending`, save
  `__state = k` and `return false`; else bind the ready value and fall through.
  `block_on(future)` (stdlib, milestone 2) loops calling `f__poll` until `Ready`.

The lowering pass slots in **after typechecking, before C emission** — a new
`src/codegen/c/async_lowering.rs` (or a pre-pass over `TypedProgram`) that
rewrites async `TypedFunction`s into the frame struct + poll fn + constructor
described above. The MVP target is the single-await, already-ready case driven
by a trivial `block_on`, proven by one runtime fixture.

### Status & exact next step

**Shipped — surface + typing:** the full surface (`@async`/`@await` lexing +
parsing) and typing (`Type::Future<T>`, async-call → future, `@await` unwrap, the
E3080/E3081 misuse codes), with parser + typechecker unit tests in
`tests/async_surface.rs`.

**Shipped — state-machine lowering (this slice).** `@async` functions whose body
is a *linear sequence of statements + optional tail*, with every `@await` at a
**top-level** position (a `VarDecl` value, a bare expression statement, or the
tail), now lower to real C and run. This covers the milestone-1 proof targets:
a leaf async fn returning a ready value **and** a chained async fn that awaits
twice, threading a local across both suspend points. Proven end to end by
`tests/zen/async_await_ready.zen` (runtime fixture
`runtime_fixtures::test_async_await_ready`); `async_await_ready_value_runs` is
un-ignored and green.

Async bodies whose awaits are **nested** inside a sub-expression, `match`/`if`
branch, or loop are still out of scope and remain gated with **E3082** (now via
`async_is_lowerable`, not a blanket "any async fn" gate). Generic async stays out
of scope — `monomorphize_types`' `Future` arm is still `unreachable!`.

**Frame ABI as actually implemented** (uniform poll-fn-pointer variant of the
sketch above — chosen so `@await` and `block_on` can drive *any* future through a
`void*` without knowing the concrete frame type):

```c
typedef bool (*zen_poll_fn)(void*, void*);   // runtime preamble

struct f__frame {
    zen_poll_fn __poll;   // uniform FIRST field — generic driving
    int __state;          // 0 = start, k = resumed after k-th await, -1 = done
    /* every param, then every local, spilled by value */
    void* __af0; void* __af1; ...  // one sub-future handle per await point
    T __ret;              // omitted for void/never
};
static bool f__poll(void* self, void* out);       // the state machine
f__frame* f(args);                                 // allocating constructor
```

- A *call* `f(args)` lowers to the constructor: `malloc` the frame, set
  `__poll = f__poll`, `__state = 0`, copy args, return the frame pointer. A
  `Future<T>` value is held in C as an opaque `void*`.
- `f__poll` is a `switch (fr->__state)`. Inside it, every spilled name `x` is
  emitted as `fr->x`. `@await e` is: evaluate `e` to a frame pointer, save the
  resume state, `case k:` poll it through `(*(zen_poll_fn*)e)(e, &__awk)`; on
  Pending `return false`, else fall through with the value in `__awk`. The
  per-await result/handle temps (`__aw<i>`/`__af<i>`) are declared at poll
  scope (C forbids a declaration right after a `case`).
- `block_on(e)` (typed `Future<T> -> T` in the checker, special-cased in the
  C backend) loops the poll until Ready. It is the compiler-provided driver
  standing in for the milestone-2 scheduler.

**Shipped — real Pending suspend/resume (milestone 2, increment 1).** The await
*handle* `__af<i>` now lives **in the frame** (a `void* __af<i>` field per await
point), initialised to `NULL` by the constructor. On a genuine Pending suspend
the poll fn takes `return false`; a later re-poll re-enters directly at the
await's `case` (the switch on `__fr->__state`), re-polls the **saved** handle,
and — because all locals are spilled into the frame too — threads earlier results
across the real suspend. Only the ready-value result temp `__aw<i>` stays at
poll-function scope (it lives for one poll). Proven end to end by
`tests/zen/async_pending_resume.zen` (runtime fixture
`runtime_fixtures::test_async_pending_resume`), which awaits futures that are
Pending for their first N polls before going Ready, driven by `block_on`'s
re-poll loop, including two real suspends in sequence with a local threaded
across them.

**Deterministic Pending source — `pending_then_ready(n, value)`.** Until the I/O
readiness mux (milestone 3) exists, there was nothing to *be* Pending on. A
compiler-provided test future fills the gap: typed `(i32, i32) -> Future<i32>`
(special-cased in `call_support.rs`, whitelisted in the resolver), lowered to a
tiny runtime frame `zen_ptr_future` (`runtime_helpers.rs`) whose poll returns
`false` for its first `n` polls then `true` with `value`. It shares the uniform
`zen_poll_fn` layout so it drives through the same `void*` path as any lowered
async frame. This is a *test* primitive, not stdlib surface.

**Shipped — cooperative scheduler primitive (milestone 2, increment 2).** A
single-threaded run-queue + round-robin driver lives in the runtime
(`runtime_helpers.rs`): `zen_scheduler` is a growable array of future-frame
handles; `zen_scheduler_run` repeatedly sweeps the queue, polling each
not-yet-Ready frame once, until every frame reports Ready — so tasks that suspend
(Pending) cooperatively interleave. Three compiler-recognised primitives expose
it (typed in `call_support.rs`, whitelisted in the resolver, lowered in
`emit.rs`):

- `scheduler_new() -> RawPtr<u8>` — a fresh empty run-queue (opaque handle);
- `scheduler_spawn(sched, fut)` — enqueue a `Future<T>` (arg 1 must be a future,
  else E3081); held opaquely as a `void*`;
- `scheduler_run(sched)` — poll every spawned future to completion.

The **mechanism** (queue + poll loop) is irreducible runtime; the **policy**
(what to spawn, when to run) is exposed to the stdlib. Proven by
`tests/zen/async_scheduler.zen` (runtime fixture
`runtime_fixtures::test_async_scheduler`): three tasks suspending 0/1/3 times all
run to completion, observed via a shared cell.

**Shipped — promoted async stdlib (milestone 2, increment 3).**
`stdlib/concurrency/async/scheduler.zen` is now a real module: a typed
`Scheduler` handle plus `scheduler()` and `run()` policy wrappers over the
runtime primitives (`@export`ed). `stdlib/concurrency/async/task.zen` exposes the
`spawn` handle-forwarding wrapper and documents the spawn/`block_on` call sites.
Driven end to end by `tests/zen/stdlib_async_scheduler.zen` (runtime fixture
`runtime_fixtures::test_stdlib_async_scheduler`).

**Not yet promotable — `async_actor.zen`, `async_helpers.zen`, `async_pool.zen`,
and a typed `spawn`/`block_on`.** All of these want to *hold or pass a future as
a value* — e.g. `spawn(s: Scheduler, t: Future<T>)`, `block_on(f: Future<T>) T`,
an actor mailbox of pending sends, an async allocator whose `alloc` returns a
future. But `Future<T>` has **no surface spelling**: there is no `AstType::Future`
and `monomorphize_types`' `Future` arm is `unreachable!`. So any future-typed
parameter is currently inexpressible in stdlib Zen, and these modules stay
placeholders. The precise unblock is to add a surface `Future<T>` type
(parser `BuiltinGenericTypeName::Future` + an `AstType::Future` + `resolve_type`
mapping + a real monomorphization arm); once a future can be named in a
signature, `spawn`/`block_on`/the actor mailbox/the async allocator all become
ordinary typed stdlib functions.

**Known limitation — control-flow splitting (increment 4, deferred).**
`async_is_lowerable` still gates (E3082) awaits nested inside a sub-expression,
`match`/`if` branch, or loop: the poll body is still a linear switch with one
`case` per top-level await. The frame already carries the resume PC (`__state`)
and the per-await handles, so the *state* side is ready; what remains is the
emission. The async poll emitter (`emit_async_statement`/`emit_async_value`) is a
**separate, simplified** path that does not reuse the normal
match/loop/block emitters, so widening it means either making the normal emitters
await-aware (and emitting `case` labels mid-construct, Duff's-device style, while
ensuring no in-branch C declaration is jumped past — all branch locals must be
frame-spilled, not emitted as C locals) or re-implementing those constructs in
the async path. That is the genuine multi-week core and was deliberately *not*
attempted half-way: a correct, tested partial that runs beats a broken whole. The
boundary is pinned by `await_inside_branch_is_still_gated_with_e3082` and
`await_nested_in_subexpression_is_gated_with_e3082` in `tests/async_surface.rs`.

**Exact next step:** widen `async_is_lowerable` + the poll emission to handle
`@await` inside `match`/`if` branches and loops — i.e. real control-flow
splitting — keeping E3082 only for genuinely-unsupported shapes (generic async,
whose `monomorphize_types` `Future` arm is still `unreachable!`). The
suspend/resume machinery (frame ABI) is now in place to support it.

Implementation lives in `src/codegen/c/async_lowering.rs` (frame/poll/ctor +
lowerability analysis), with the `block_on` driver and scheduler primitive
lowering in `src/codegen/c/emit.rs`, and the `zen_poll_fn` typedef, the
`zen_ptr_future` test future, and the `zen_scheduler` run-queue in
`src/codegen/c/types/runtime_helpers.rs`. Scheduler/test-future *typing* is in
`src/typechecker/expressions/call_support.rs`; resolver whitelisting in
`src/resolver/local_validation.rs`.
