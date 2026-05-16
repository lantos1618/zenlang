use super::*;

#[test]
fn enum_match_missing_variant_is_error() {
    let program = parse_program(
        r#"
Color: Red, Green, Blue

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Green { "green" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-exhaustive enum match should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-exhaustive match on `Color`: missing `Blue`")),
        "expected non-exhaustive enum diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_duplicate_variant_is_error() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Red { "again" }
        | Green { "green" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("duplicate enum match arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate match arm for `Color.Red`")),
        "expected duplicate enum arm diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_unknown_variant_is_error() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Blue { "blue" }
        | Green { "green" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown enum match arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("enum `Color` has no variant `Blue`")),
        "expected unknown enum arm diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_payload_shape_is_checked() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

describe = (m: Maybe) StaticString {
    m ?
        | Some { "some" }
        | None(value) { "none" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("enum match payload shape should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("match arm `Maybe.Some` requires a payload")),
        "expected missing payload diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("match arm `Maybe.None` does not accept a payload")),
        "expected forbidden payload diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_wildcard_after_all_variants_is_redundant() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Green { "green" }
        | _ { "fallback" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("redundant enum wildcard arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("redundant wildcard match arm")),
        "expected redundant wildcard diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_variant_after_wildcard_is_redundant() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | _ { "fallback" }
        | Red { "red" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("enum variant after wildcard should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("redundant match arm for `Color.Red`")),
        "expected redundant enum arm diagnostic, got {errors:?}"
    );
}

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
