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

**Known limitation — real Pending suspend.** The MVP only exercises
*already-ready* futures, which never take the `return false` path. The await
*handle* temps (`__af<i>`) live at poll-function scope, not in the frame, so they
are **not** preserved across a genuine poll/return/re-poll cycle: a future that
actually returns Pending would lose its handle on resume. This is acceptable for
milestone 1 (there is nothing to be Pending *on* until the I/O mux of
milestone 3, and no scheduler to re-poll until milestone 2), but it is the first
thing to fix when a real Pending source lands: spill `__af<i>` (and the resume
PC for awaits inside control flow) into the frame, and re-derive them on resume.

**Exact next step:** extend `async_is_lowerable` + the poll emission to handle
`@await` inside `match`/`if` branches and loops — i.e. real control-flow
splitting — and move the await handles into the frame so a Pending re-poll
resumes correctly. That, plus the milestone-2 cooperative scheduler (which gives
something to actually suspend for), turns the single-threaded skeleton into a
usable coroutine runtime.

Implementation lives in `src/codegen/c/async_lowering.rs` (frame/poll/ctor +
lowerability analysis), with the `block_on` driver in `src/codegen/c/emit.rs`
and the `zen_poll_fn` typedef in `src/codegen/c/types/runtime_helpers.rs`.
