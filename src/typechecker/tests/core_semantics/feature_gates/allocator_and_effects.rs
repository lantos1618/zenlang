use super::*;

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
fn sync_and_async_typed_allocator_modes_are_rejected_as_gated_not_unknown() {
    let program = parse_program(
        r#"
use_sync_allocator = (allocator: Allocator<i32, Sync>) void { }
use_async_allocator = (allocator: Allocator<i32, Async>) void { }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc.check_program(&program).expect_err(
        "typed allocator Sync/Async modes should stay gated until allocator semantics exist",
    );

    let allocator_gate_count = err
        .iter()
        .filter(|diagnostic| diagnostic.message.contains("typed allocators are gated"))
        .count();
    assert_eq!(
        allocator_gate_count, 2,
        "expected Sync and Async allocator annotations to both report allocator gates, got {err:?}"
    );
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown type symbol")),
        "allocator/effect mode gates should not be reported as ordinary unknown types, got {err:?}"
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
