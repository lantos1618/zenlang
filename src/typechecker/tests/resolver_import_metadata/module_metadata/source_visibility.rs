use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_module_symbols() {
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
    symbols.set_public_for_test(Namespace::Module, "std", true);
    symbols.set_import_source_for_test(Namespace::Module, "std", Some("other".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver module metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has visibility public, expected private")),
        "expected resolver module visibility diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has source 'other', expected none")),
        "expected resolver module source diagnostic, got {err:?}"
    );
}
