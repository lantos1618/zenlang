use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_method_signatures() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "StaticString".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior method signature mismatch should fail");

    let expected = "resolver behavior symbol 'Json' has methods '(encode(Self) StaticString)', expected '(encode(Self) T)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior method signature diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_function_type_method_signatures()
{
    let program = parse_program(
        r#"
Mapper<T>: behavior {
    map: (Self, (T) T) (T) T
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
            vec!["Self".to_string(), "T".to_string()],
            "(T) T".to_string(),
        )]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior function type method mismatch should fail");

    let expected = "resolver behavior symbol 'Mapper' has methods '(map(Self, T) (T) T)', expected '(map(Self, (T) T) (T) T)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior function type method diagnostic, got {err:?}"
    );
}
