use super::*;

#[test]
fn behavior_impl_with_required_method_passes() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.implements(Json) {
    to_json = (value: Point) StaticString { "point" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("valid behavior impl should typecheck");
}

#[test]
fn behavior_impl_required_method_accepts_self_parameter() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.implements(Json) {
    to_json = (value: Self) StaticString { "point" }
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("behavior impl method may use Self for the impl target");
}

#[test]
fn behavior_impl_missing_required_method_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.implements(Json) {
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("missing behavior method should fail");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "type `Point` implementation of `Json` is missing required method `to_json`"
        )),
        "expected missing behavior method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_can_omit_default_method() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString { "{}" }
}

Point.implements(Json) {
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("behavior impl may omit a method with a default body");
}

#[test]
fn behavior_impl_duplicate_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.implements(Json) {
    to_json = (value: Point) StaticString { "point" }
}

Point.implements(Json) {
    to_json = (value: Point) StaticString { "point" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("duplicate behavior impl should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("duplicate implementation of behavior `Json` for type `Point`")),
        "expected duplicate behavior impl diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_overlapping_inherited_behavior_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)

Point.implements(Json) {
    to_json = (value: Point) StaticString { "point" }
}

Point.implements(PrettyJson) {
    to_json = (value: Point) StaticString { "point" }
    pretty = (value: Point) StaticString { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("overlapping inherited behavior impl should fail");
    assert!(
        errors.iter().any(|d| {
            d.message.contains(
                "overlapping implementations of behaviors `Json` and `PrettyJson` for type `Point`",
            )
        }),
        "expected overlapping behavior impl diagnostic, got {errors:?}"
    );
}
