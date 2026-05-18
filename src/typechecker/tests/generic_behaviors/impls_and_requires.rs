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
fn behavior_impl_generic_behavior_without_type_args_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior impl without type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 0")),
        "expected generic behavior impl arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_nongeneric_behavior_type_args_are_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json<i32>) {
    encode = (value: Point) StaticString { "point" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("non-generic behavior impl with type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic behavior `Json` does not accept type arguments")),
        "expected non-generic behavior impl type-argument diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("generic behavior `Json` expects 0")),
        "non-generic behavior impl should not use generic arity wording, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_with_type_args_passes_requires() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior impl should satisfy matching generic requires");
}

#[test]
fn behavior_impl_generic_behavior_type_arg_bound_failure_is_error() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { value }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior type argument bound should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json<Point>` required by `T`")),
        "expected generic behavior type argument bound diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Json<Point>) {
    encode = (value: Point) Point { value }
}

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { value }
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior type argument bound should pass when satisfied");
}

#[test]
fn behavior_impl_generic_behavior_substitutes_method_signature() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) i32 { 1 }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior impl return mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("method `encode` for behavior `Json_StaticString` expects return `StaticString`, found `i32`")),
        "expected substituted behavior method return diagnostic, got {errors:?}"
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
