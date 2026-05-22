use super::*;

#[test]
fn check_program_with_symbols_requires_resolver_parameter_locals() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Local, "a");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver parameter local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing local symbol 'a'")),
        "expected missing resolver parameter local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_parameter_local_mutability() {
    let program = parse_program(
        r#"
add = (mut a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_local_mutability_for_test("a", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver parameter local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has mutability immutable, expected mutable")),
        "expected resolver parameter local mutability diagnostic, got {err:?}"
    );
}
