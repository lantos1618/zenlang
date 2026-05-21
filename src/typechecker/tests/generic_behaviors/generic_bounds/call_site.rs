use super::*;

#[test]
fn generic_behavior_bound_accepts_type_with_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.implements(Json) {
    to_json = (value: Point) StaticString { "point" }
}

encode<T: Json> = (value: T) StaticString {
    "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("type with behavior impl should satisfy generic bound");
}

#[test]
fn generic_behavior_bound_accepts_inherited_behavior_impl() {
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

encode<T: Json> = (value: T) StaticString {
    "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    0
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("child behavior impl should satisfy inherited generic bound");
}

#[test]
fn generic_behavior_bound_rejects_type_without_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

encode<T: Json> = (value: T) StaticString {
    "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("type without behavior impl should not satisfy generic bound");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json`")),
        "expected missing generic bound impl diagnostic, got {errors:?}"
    );
}
