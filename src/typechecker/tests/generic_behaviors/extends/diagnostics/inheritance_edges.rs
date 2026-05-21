use super::super::*;

#[test]
fn behavior_extends_cycle_is_error() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
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
    to_json: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
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
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)
PrettyJson.extends(Json<StaticString>)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("duplicate specialized behavior inheritance edge should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("duplicate behavior inheritance `PrettyJson.extends(Json<StaticString>)`")
        }),
        "expected duplicate generic behavior inheritance diagnostic, got {errors:?}"
    );
}
