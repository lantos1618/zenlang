use super::*;

#[test]
fn check_program_with_symbols_requires_resolver_enum_variants() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Variant, "Some");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing resolver enum variant symbols should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing variant symbol 'Some'")),
        "expected missing enum variant symbol diagnostic, got {err:?}"
    );
}
