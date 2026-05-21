use super::super::*;

#[test]
fn behavior_extends_generic_parent_without_type_args_is_error() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
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
    encode: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
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
