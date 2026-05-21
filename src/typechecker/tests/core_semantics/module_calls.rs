use super::*;

#[test]
fn unknown_root_std_module_call_is_error() {
    let program = parse_program(
        r#"
{ io } = std

main = () void {
    io.nope("bad")
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown std module function should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("undefined module function `io.nope`")),
        "expected undefined module function diagnostic, got {errors:?}"
    );
}

#[test]
fn known_root_std_runtime_standins_remain_allowed() {
    let program = parse_program(
        r#"
{ io } = std

main = () void {
    io.print("hello")
    io.println("world")
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("temporary root std io stand-ins should typecheck");
}
