use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_module_absent_declaration_metadata() {
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
    symbols.set_parameter_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Module, "std", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver module declaration metadata should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has parameter count metadata, expected none")),
        "expected resolver module parameter metadata diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has return type metadata, expected none")),
        "expected resolver module return metadata diagnostic, got {err:?}"
    );
}
