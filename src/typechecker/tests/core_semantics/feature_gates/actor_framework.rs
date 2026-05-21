use super::*;

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
fn bare_actor_framework_types_are_rejected_as_gated_not_unknown() {
    let program = parse_program(
        r#"
use_actor = (actor: Actor) void { }
use_ref = (ref: ActorRef) void { }
use_mailbox = (mailbox: Mailbox) void { }
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("bare actor framework types should stay gated until std actor semantics exist");

    for expected in [
        "`Actor` framework type is gated",
        "`ActorRef` framework type is gated",
        "`Mailbox` framework type is gated",
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
        "bare actor framework gates should not be reported as ordinary unknown types, got {err:?}"
    );
}
