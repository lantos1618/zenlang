use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_import_absent_declaration_metadata() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Import, "io", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import declaration metadata should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has parameter count metadata, expected none")),
        "expected resolver import parameter metadata diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has return type metadata, expected none")),
        "expected resolver import return metadata diagnostic, got {err:?}"
    );
}
