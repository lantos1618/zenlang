use super::super::*;

#[test]
fn behavior_extends_conflicting_method_signature_is_error() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) StaticString
}

PrettyJson: behavior {
    to_json: (Self) i32
}

PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("conflicting inherited behavior method should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("conflicting behavior method `to_json` inherited by `PrettyJson`")
        }),
        "expected conflicting inherited behavior method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_signature_mismatch_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.implements(Json) {
    to_json = (value: i32) i32 { value }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("behavior impl signature mismatch should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("parameter 1 for method `to_json`")),
        "expected behavior parameter mismatch diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("expects return `StaticString`, found `i32`")),
        "expected behavior return mismatch diagnostic, got {errors:?}"
    );
}
