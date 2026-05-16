use super::*;

#[test]
fn behavior_extends_requires_parent_methods() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "pretty" }
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
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    to_json = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
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
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "pretty" }
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
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}

Point.requires(Json<str>)
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
fn behavior_impl_generic_parent_overlap_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.implements(PrettyJson) {
    encode = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("specialized parent and child behavior impls should overlap");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "overlapping implementations of behaviors `Json_str` and `PrettyJson` for type `Point`"
        )),
        "expected specialized behavior impl overlap diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_distinct_generic_specializations_do_not_overlap() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.implements(Json<i32>) {
    encode = (value: Point) i32 { value.x }
}

Point.requires(Json<str>)
Point.requires(Json<i32>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("distinct behavior specializations should not overlap");
}

#[test]
fn behavior_extends_cycle_is_error() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

Json.extends(PrettyJson)
PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("cyclic behavior inheritance should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("behavior inheritance cycle")),
        "expected behavior inheritance cycle diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_duplicate_parent_is_error() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("duplicate behavior inheritance edge should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("duplicate behavior inheritance `PrettyJson.extends(Json)`")
        }),
        "expected duplicate behavior inheritance diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_duplicate_generic_parent_is_error() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
PrettyJson.extends(Json<str>)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("duplicate specialized behavior inheritance edge should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("duplicate behavior inheritance `PrettyJson.extends(Json<str>)`")
        }),
        "expected duplicate generic behavior inheritance diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_generic_parent_without_type_args_is_error() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior extends parent without type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 0")),
        "expected generic behavior extends parent arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_conflicting_method_signature_is_error() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str
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
    to_json: (Self) str
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
        errors
            .iter()
            .any(|d| d.message.contains("expects return `str`, found `i32`")),
        "expected behavior return mismatch diagnostic, got {errors:?}"
    );
}
