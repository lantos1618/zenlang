use super::*;

#[test]
fn syscall_intrinsics_are_rejected_as_host_effect_gates() {
    let program = parse_program(
        r#"
main = () void {
    @builtin.syscall0(1)
    @builtin.syscall1(1, 2)
    @builtin.syscall2(1, 2, 3)
    @builtin.syscall3(1, 2, 3, 4)
    @builtin.syscall4(1, 2, 3, 4, 5)
    @builtin.syscall5(1, 2, 3, 4, 5, 6)
    @builtin.syscall6(1, 2, 3, 4, 5, 6, 7)
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("syscall intrinsics should stay gated until host effects exist");

    for expected in [
        "syscall0 is gated",
        "syscall1 is gated",
        "syscall2 is gated",
        "syscall3 is gated",
        "syscall4 is gated",
        "syscall5 is gated",
        "syscall6 is gated",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown function `@builtin.syscall")),
        "syscall gates should not be reported as ordinary unknown builtins, got {err:?}"
    );
}
