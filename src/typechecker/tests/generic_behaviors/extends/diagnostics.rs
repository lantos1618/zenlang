use super::*;

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
fn behavior_extends_nongeneric_parent_type_args_are_error() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<i32>)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("non-generic behavior extends parent with type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic behavior `Json` does not accept type arguments")),
        "expected non-generic behavior extends parent type-argument diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("generic behavior `Json` expects 0")),
        "non-generic behavior extends parent should not use generic arity wording, got {errors:?}"
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
