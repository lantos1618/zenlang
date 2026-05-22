use super::*;

#[test]
fn atomic_intrinsics_are_rejected_as_effect_gates() {
    let program = parse_program(
        r#"
main = () void {
    @builtin.atomic_load(0)
    @builtin.atomic_store(0, 1)
    @builtin.atomic_add(0, 1)
    @builtin.atomic_sub(0, 1)
    @builtin.atomic_cas(0, 1, 2)
    @builtin.atomic_xchg(0, 1)
    @builtin.fence()
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("atomic intrinsics should stay gated until memory-order semantics exist");

    for expected in [
        "atomic load is gated",
        "atomic store is gated",
        "atomic add is gated",
        "atomic subtract is gated",
        "atomic compare-and-swap is gated",
        "atomic exchange is gated",
        "atomic fence is gated",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown function `@builtin.atomic_")),
        "atomic gates should not be reported as ordinary unknown builtins, got {err:?}"
    );
}
