use super::*;

#[test]
fn range_expression_is_rejected_until_range_type_exists() {
    let program = parse_program(
        r#"
main = () i32 {
    1..3
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("range expressions should be gated until range typing exists");

    assert!(
        err.iter()
            .any(|d| d.message.contains("range expressions are not implemented")),
        "expected range diagnostic, got {err:?}"
    );
}

#[test]
fn result_raise_is_rejected_until_propagation_lowering_exists() {
    let program = parse_program(
        r#"
Result<T, E>:
    Ok(T),
    Err(E)

main = () i32 {
    value = Result<i32, StaticString>.Ok(1)
    value.raise()
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("raise propagation should stay gated until lowering exists");

    assert!(
        err.iter()
            .any(|d| d.message.contains("`.raise()` is gated")),
        "expected raise gate diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .all(|d| !d.message.contains("has no method `raise`")),
        "raise gate should not be reported as an ordinary missing method, got {err:?}"
    );
}

#[test]
fn effect_await_is_rejected_until_async_lowering_exists() {
    let program = parse_program(
        r#"
Task<T>: {
    value: T
}

main = () i32 {
    task = Task<i32> { value: 1 }
    task.await()
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("await should stay gated until effect checking and task lowering exist");

    assert!(
        err.iter()
            .any(|d| d.message.contains("`.await()` is gated")),
        "expected await gate diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .all(|d| !d.message.contains("has no method `await`")),
        "await gate should not be reported as an ordinary missing method, got {err:?}"
    );
}

#[test]
fn async_scheduler_intrinsics_are_rejected_as_gated_not_unknown() {
    let program = parse_program(
        r#"
main = () void {
    @builtin.async_enqueue(1)
    @builtin.async_yield()
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc.check_program(&program).expect_err(
        "async scheduler intrinsics should stay gated until effect checking and task lowering exist",
    );

    for expected in ["async task enqueue is gated", "async yield is gated"] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown function `@builtin.async_")),
        "async scheduler gates should not be reported as ordinary unknown builtins, got {err:?}"
    );
}

#[test]
fn typed_allocator_type_is_rejected_as_gated_not_unknown() {
    let program = parse_program(
        r#"
main = (allocator: Allocator<i32, Sync>) void { }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("typed allocator types should stay gated until allocator semantics exist");

    assert!(
        err.iter()
            .any(|d| d.message.contains("typed allocators are gated")),
        "expected typed allocator gate diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown type symbol")),
        "allocator/effect gate should not be reported as ordinary unknown types, got {err:?}"
    );
}

#[test]
fn raw_memory_intrinsics_are_rejected_as_allocator_gates() {
    let program = parse_program(
        r#"
main = () void {
    ptr = @builtin.raw_allocate(8)
    @builtin.raw_deallocate(ptr, 8)
    @builtin.raw_reallocate(ptr, 8, 16)
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("raw memory intrinsics should stay gated until allocator semantics exist");

    for expected in [
        "raw allocation is gated",
        "raw deallocation is gated",
        "raw reallocation is gated",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown function `@builtin.raw_")),
        "raw memory gates should not be reported as ordinary unknown builtins, got {err:?}"
    );
}

#[test]
fn dynamic_string_type_is_rejected_as_allocator_backed_gate() {
    let program = parse_program(
        r#"
main = (value: String) void { }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("dynamic String should stay gated until allocator ownership exists");

    assert!(
        err.iter().any(|d| d.message.contains("`String` is gated")),
        "expected dynamic String gate diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown type symbol")),
        "dynamic String gate should not be reported as an ordinary unknown type, got {err:?}"
    );
}

#[test]
fn sync_async_effect_modes_are_rejected_as_gated_not_unknown() {
    let program = parse_program(
        r#"
use_sync = (mode: Sync) void { }
use_async = (mode: Async) void { }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("effect mode types should stay gated until effect semantics exist");

    for expected in [
        "`Sync` effect mode is gated",
        "`Async` effect mode is gated",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown type symbol")),
        "effect mode gate should not be reported as ordinary unknown types, got {err:?}"
    );
}

#[test]
fn actor_framework_types_are_rejected_as_gated_not_unknown() {
    let program = parse_program(
        r#"
use_actor = (actor: Actor<i32>) void { }
use_ref = (ref: ActorRef<i32>) void { }
use_mailbox = (mailbox: Mailbox<i32>) void { }
use_supervisor = (supervisor: Supervisor) void { }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("actor framework types should stay gated until std actor semantics exist");

    for expected in [
        "`Actor` framework type is gated",
        "`ActorRef` framework type is gated",
        "`Mailbox` framework type is gated",
        "`Supervisor` framework type is gated",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown type symbol")),
        "actor framework gate should not be reported as ordinary unknown types, got {err:?}"
    );
}

#[test]
fn comptime_type_match_intrinsic_is_rejected_as_gated_not_unknown() {
    let program = parse_program(
        r#"
Point: { x: i32 }

main = () void {
    @builtin.type_match<Point>()
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc.check_program(&program).expect_err(
        "comptime type matching should stay gated until typed metadata lowering exists",
    );

    assert!(
        err.iter()
            .any(|d| d.message.contains("comptime type matching is gated")),
        "expected type-match gate diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown function `@builtin.type_match`")),
        "type-match gate should not be reported as an ordinary unknown builtin, got {err:?}"
    );
}
