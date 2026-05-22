use super::*;

#[test]
fn bool_match_missing_arm_is_error_for_value_match() {
    let program = parse_program(
        r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "yes" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-exhaustive boolean value match should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-exhaustive bool match: missing `false`")),
        "expected non-exhaustive bool diagnostic, got {errors:?}"
    );
}

#[test]
fn bool_match_duplicate_arm_is_error() {
    let program = parse_program(
        r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "yes" }
        | true { "again" }
        | false { "no" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("duplicate boolean match arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate match arm for `true`")),
        "expected duplicate bool arm diagnostic, got {errors:?}"
    );
}

#[test]
fn match_arm_return_does_not_force_never_result_type() {
    let program = parse_program(
        r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "early" }
        | false { "late" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_program(&program)
        .expect("returning arm should not force match type to never");
    let body = &typed.functions[0].body;
    assert_eq!(body.ty, Type::Str);
}
