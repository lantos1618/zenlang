use super::*;

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

#[test]
fn primitive_and_enum_type_match_intrinsics_are_rejected_as_gated_not_unknown() {
    let program = parse_program(
        r#"
Choice:
    First,
    Second

main = () void {
    @builtin.type_match<i32>()
    @builtin.type_match<Choice>()
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc.check_program(&program).expect_err(
        "primitive and enum comptime type matching should stay gated until typed metadata lowering exists",
    );

    let type_match_gate_count = err
        .iter()
        .filter(|d| d.message.contains("comptime type matching is gated"))
        .count();
    assert_eq!(
        type_match_gate_count, 2,
        "expected primitive and enum type-match calls to both report gates, got {err:?}"
    );
    assert!(
        err.iter()
            .all(|d| !d.message.contains("unknown function `@builtin.type_match`")),
        "primitive/enum type-match gates should not be reported as ordinary unknown builtins, got {err:?}"
    );
}
