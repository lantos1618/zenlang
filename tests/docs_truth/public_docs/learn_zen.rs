use super::super::*;

#[test]
fn learn_zen_guide_covers_core_tour_and_gated_previews() {
    let guide = read("docs/learn_zen_in_y_minutes.md");

    for required in [
        "# Learn Zen In Y Minutes",
        "Use this page as the quick language tour",
        "Stable examples are forms to copy into\nsource today",
        "Preview examples are intentionally Zen-shaped",
        "Declarations are prefix-first",
        "Need | Spell it like this",
        "Mutable inferred local",
        "Function | `name = (arg: Type) ResultType { final_expression }`",
        "Method | `Type.method = (self: Type) ResultType { final_expression }`",
        "Loop | `loop((l) { ... l.next() ... l.done() ... })`",
        "Implementation | `Type.implements(Behavior) { ... }`",
        "Requirement | `Type.requires(Behavior)`",
        "Inheritance | `ChildBehavior.extends(ParentBehavior)`",
        "Zen does not use a `return` keyword",
        "put the value at the end of the block",
        "String literals are `StaticString`, not allocator-backed strings",
        "`StaticString` is baked into the program",
        "static bytes plus a fixed byte\ncount known after compilation",
        "does not allocate, resize, free, or transfer heap\nownership",
        "`String<A>` is preview syntax for owned runtime text",
        "must carry allocator state",
        "A literal such as `\"Zen\"` never silently becomes `String<A>`",
        "Runtime text\nconstruction belongs on an allocator-aware API",
        "value.method(args)",
        "method(value, args)",
        "call-site spellings for the\nsame attached function",
        "## Result And Error Handling",
        "There are no exceptions and no null",
        "Result<T, E>",
        "Option<T>",
        "Nested generic types are written directly",
        "Display: behavior",
        "show<T: Display>",
        "Point.implements(Display)",
        "Point.requires(Display)",
        "PrettyDisplay.extends(Display)",
        "`impl Type for Behavior`",
        "There is no `impl Type for Behavior` spelling",
        "## Loops",
        "Zen has one loop entry form",
        "The loop handle is compiler-owned",
        "closed loop-control\nverbs for that handle",
        "not arbitrary user methods and not stringly names",
        "Counted loop",
        "sum_to = (limit: i32) i32",
        "Nested loop exit",
        "outer.done()",
        "inner.next()",
        "UFC loop control",
        "loop((l) {\n    done(l)\n    next(l)\n})",
        "There is no `while`, `for`, `break`, `continue`, suffix loop, or hidden loop\nresult channel",
        "## Defer",
        "## Imports And Modules",
        "## Memory And Ownership",
        "does not hide heap allocation",
        "OwnedBytes<T, A>",
        "Pointer, length, capacity, and allocator travel together",
        "## Sync, Async, And Allocator Preview",
        "`Sync` and `Async` are effect modes in type surfaces",
        "They are not source\nkeywords and there is no `async fn` spelling",
        "`async fn`",
        "Sync work returns checked data now",
        "Async work returns task-shaped data",
        "Allocator<T, Sync>",
        "Allocator<T, Async>",
        "Read the outer type first",
        "`Result<T, E>` | checked data is available now",
        "`Task<Result<T, E>>` | checked data belongs to scheduled work",
        "`String<A>` | owned dynamic bytes plus allocator ownership",
        "There is no hidden conversion between sync and async allocation",
        "make_buffer<T, A: Allocator<T, Sync>>",
        "@builtin.raw_allocate",
        "@builtin.raw_deallocate",
        "@builtin.raw_reallocate",
        "## Pointer, Slice, Array, Actor, And Comptime Preview",
        "RawPtr<T>",
        "`Ptr<T>`, `MutPtr<T>`, `Slice<T>`, and `[T; N]`",
        "raw pointer offset",
        "comptime type matching",
        "actor framework",
        "gated design",
        "## Translation Cheat Sheet",
        "`while condition { ... }`",
        "`for item in items { ... }`",
        "keyword exit value",
        "growable owned text",
        "## One Page Example",
        "behavior implementations, bounded generics, expression-oriented control flow,\nstatic text, and explicit loop control",
        "docs/V1_SPEC.md",
    ] {
        assert!(
            guide.contains(required),
            "Learn guide is missing expected tour or gated-preview text: {required}"
        );
    }

    for stale in ["## Impl Blocks", "Type.impl =", ".impl = {", "impl blocks:"] {
        assert!(
            !guide.contains(stale),
            "Learn guide should avoid teaching non-behavior impl-block syntax as the public tutorial path: {stale}"
        );
    }

    assert!(
        guide.lines().count() <= 360,
        "Learn guide should stay compact; move detailed status or evidence to phase docs and tests"
    );
}
