use super::*;

mod absent_metadata;
mod module_metadata;

#[test]
fn check_program_with_symbols_validates_resolver_import_sources() {
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
    symbols.set_import_source_for_test(Namespace::Import, "io", Some("other".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import source mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has source 'other', expected 'std'")),
        "expected resolver import source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_import_visibility() {
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
    symbols.set_public_for_test(Namespace::Import, "io", true);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has visibility public, expected private")),
        "expected resolver import visibility diagnostic, got {err:?}"
    );
}
