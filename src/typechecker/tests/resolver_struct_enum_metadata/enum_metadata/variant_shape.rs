use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_visibility() {
    let program = parse_program(
        r#"
pub Option<T>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Variant, "Some", false);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has visibility private, expected public")),
        "expected resolver enum variant visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Option", Some(vec!["Some".to_string()]));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant names mismatch should fail");

    let expected = "resolver type symbol 'Option' has variants '(Some)', expected '(Some, None)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver enum variant names diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_owner_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_owner_name_for_test(Namespace::Variant, "Some", Some("Result".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant owner mismatch should fail");

    let expected = "resolver variant symbol 'Some' has owner 'Result', expected 'Option'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver enum variant owner diagnostic, got {err:?}"
    );
}
