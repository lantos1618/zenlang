use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_generic_enum_function_type_payloads() {
    let program = parse_program(
        r#"
Callback<T>: Wrap((T) T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Wrap",
        Some("T".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic enum function type payload mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Wrap' has payload type 'T', expected '(T) T'")),
        "expected resolver generic enum function type payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_enum_payload_types() {
    let program = parse_program(
        r#"
Result<T, E>: Ok(T), Err(E)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Err",
        Some("T".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic enum payload mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Err' has payload type 'T', expected 'E'")),
        "expected resolver generic enum payload diagnostic, got {err:?}"
    );
}
