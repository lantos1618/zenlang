use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_behavior_method_signatures() {
    let program = parse_program(
        r#"
Serializable: behavior {
    encode: (Self, i32) StaticString
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Behavior,
        "Serializable",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string(), "bool".to_string()],
            "StaticString".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior method signature mismatch should fail");

    let expected = "resolver behavior symbol 'Serializable' has methods '(encode(Self, bool) StaticString)', expected '(encode(Self, i32) StaticString)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver behavior method signature diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_function_type_method_signatures() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Behavior,
        "Mapper",
        Some(vec![(
            "map".to_string(),
            vec!["Self".to_string(), "i32".to_string()],
            "(i32) i32".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior function type method signature mismatch should fail");

    let expected = "resolver behavior symbol 'Mapper' has methods '(map(Self, i32) (i32) i32)', expected '(map(Self, (i32) i32) (i32) i32)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver behavior function type method signature diagnostic, got {err:?}"
    );
}
