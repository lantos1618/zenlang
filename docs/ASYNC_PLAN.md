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

**Shipped (this slice):** the full surface (`@async`/`@await` lexing + parsing)
and typing (`Type::Future<T>`, async-call → future, `@await` unwrap, the E3080/
E3081 misuse codes), with parser + typechecker unit tests in
`tests/async_surface.rs`. Programs that *define* an `@async` function are gated
with **E3082** before codegen, so the build never emits a half-lowered async fn.
The `Future`/`Await` arms in the C backend and `monomorphize_types` are therefore
`unreachable!` for accepted programs (reached only if the E3082 gate is removed).

**Next step (start of the lowering slice), in order:**

1. Thread `is_async` from `Declaration::Function` onto `TypedFunction` (add the
   field in `src/ast/typed.rs`, set it in
   `src/typechecker/expressions/function_checking.rs::check_function`). Right now
   the flag is consumed only to type the body and to drive the E3082 gate; the
   lowering pass needs it on the typed node.
2. Add `src/codegen/c/async_lowering.rs`: a function
   `lower_async(program: &mut TypedProgram)` that, for each `is_async`
   `TypedFunction`, emits the frame struct (as a `TypedTypeDef`), the poll fn,
   and the constructor per the ABI above. For the MVP, support exactly one
   `@await` whose operand is a ready future: no live-local spilling across the
   suspend is required yet (the value is ready), so state `0` polls the inner
   future, writes `__ret`, sets `__state = -1`, returns `true`.
3. Provide a minimal C `block_on` (emit it inline in the runtime-helpers
   preamble, `src/codegen/c/types/runtime_helpers.rs`) that loops `f__poll`
   until it returns `true` and yields `*out`. Until the stdlib scheduler
   (milestone 2) exists, this is a compiler-emitted helper.
4. Remove the E3082 gate in `src/typechecker/program_checking.rs` and flip the
   `unreachable!` arms to real emission. Replace the `#[ignore]` on
   `async_await_ready_value_runs` in `tests/async_surface.rs` with a real
   runtime fixture (`tests/zen/<name>.zen` + `expected/<name>.expected` + a
   `#[test]` in `tests/integration/runtime_fixtures.rs`).

Doing 1–4 as one coherent change proves the irreducible piece (suspend/resume
frame + poll) end to end for the simplest case; N-await and live-local spilling
across real suspends follow as the next milestone-1 increments.
