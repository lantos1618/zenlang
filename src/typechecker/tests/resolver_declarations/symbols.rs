use super::*;

#[test]
fn check_program_with_symbols_requires_resolver_declarations() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let empty_symbols = SymbolTable::default();
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &empty_symbols)
        .expect_err("missing resolver symbols should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing value symbol 'main'")),
        "expected missing resolver symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_declarations() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let symbols_program = parse_program(
        r#"
main = () i32 { 0 }
extra = () i32 { 1 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver declarations should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra value symbol 'extra'")),
        "expected extra resolver symbol diagnostic, got {err:?}"
    );
}
