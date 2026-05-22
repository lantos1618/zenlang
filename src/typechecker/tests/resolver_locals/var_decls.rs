use super::*;

#[test]
fn check_program_with_symbols_requires_resolver_var_decl_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    value = 1
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "value");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver var local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'value'")),
        "expected missing resolver var local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_var_decl_local_mutability() {
    let program = parse_program(
        r#"
main = () i32 {
    value ::= 1
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_local_mutability_for_test("value", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver var local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'value' has mutability immutable, expected mutable")),
        "expected resolver var local mutability diagnostic, got {err:?}"
    );
}
