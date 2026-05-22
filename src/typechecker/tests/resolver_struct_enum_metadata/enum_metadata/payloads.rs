use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_payload_counts() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_count_for_test(Namespace::Variant, "Some", Some(0));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant payload count mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has payload count 0, expected 1")),
        "expected resolver enum variant payload count diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_payload_types() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Some",
        Some("bool".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant payload type mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has payload type 'bool', expected 'i32'")),
        "expected resolver enum variant payload type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_function_type_payloads() {
    let program = parse_program(
        r#"
Callback: Wrap((i32) i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Wrap",
        Some("i32".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum function type payload mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver variant symbol 'Wrap' has payload type 'i32', expected '(i32) i32'"
        )),
        "expected resolver enum function type payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_typed_payload_metadata() {
    let program = parse_program(
        r#"
Callback: Wrap((i32) i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_for_test(Namespace::Variant, "Wrap", Some(AstType::I32));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed enum payload metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver variant symbol 'Wrap' has typed payload type 'i32', expected '(i32) i32'"
        )),
        "expected resolver typed enum payload diagnostic, got {err:?}"
    );
}
