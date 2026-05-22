use super::*;

#[test]
fn behavior_extends_requires_parent_methods() {
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

Point.implements(PrettyJson) {
    pretty = (value: Point) StaticString { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("extended behavior should require parent methods");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "type `Point` implementation of `PrettyJson` is missing required method `to_json`"
        )),
        "expected inherited missing method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_impl_satisfies_parent_requires() {
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

Point.implements(PrettyJson) {
    to_json = (value: Point) StaticString { "point" }
    pretty = (value: Point) StaticString { "pretty" }
}

Point.requires(Json)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("implementation of child behavior should satisfy parent requires");
}
