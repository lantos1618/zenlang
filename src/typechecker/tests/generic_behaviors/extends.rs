use super::*;

mod diagnostics;

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

#[test]
fn behavior_extends_generic_parent_requires_substituted_methods() {
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

Point.implements(PrettyJson) {
    pretty = (value: Point) StaticString { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic parent method should be required with substituted signature");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "type `Point` implementation of `PrettyJson` is missing required method `encode`"
        )),
        "expected inherited generic parent missing method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_generic_parent_satisfies_specialized_requires() {
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

Point.implements(PrettyJson) {
    encode = (value: Point) StaticString { "point" }
    pretty = (value: Point) StaticString { "pretty" }
}

Point.requires(Json<StaticString>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("child behavior impl should satisfy specialized generic parent requires");
}

#[test]
fn behavior_extends_generic_parent_accepts_child_type_parameter_arg() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior parent should accept child type parameter args");
}

#[test]
fn behavior_impl_distinct_generic_specializations_do_not_overlap() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(Json<i32>) {
    encode = (value: Point) i32 { value.x }
}

Point.requires(Json<StaticString>)
Point.requires(Json<i32>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("distinct behavior specializations should not overlap");
}
