use super::super::*;

#[test]
fn behavior_impl_generic_parent_overlap_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(PrettyJson) {
    encode = (value: Point) StaticString { "point" }
    pretty = (value: Point) StaticString { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("specialized parent and child behavior impls should overlap");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "overlapping implementations of behaviors `Json_StaticString` and `PrettyJson` for type `Point`"
        )),
        "expected specialized behavior impl overlap diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_transitive_generic_parent_overlap_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

CompactJson: behavior {
    compact: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

CompactJson.extends(Json<StaticString>)
PrettyJson.extends(CompactJson)

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(PrettyJson) {
    encode = (value: Point) StaticString { "point" }
    compact = (value: Point) StaticString { "compact" }
    pretty = (value: Point) StaticString { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("transitive specialized parent and child behavior impls should overlap");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "overlapping implementations of behaviors `Json_StaticString` and `PrettyJson` for type `Point`"
        )),
        "expected transitive specialized behavior impl overlap diagnostic, got {errors:?}"
    );
}
