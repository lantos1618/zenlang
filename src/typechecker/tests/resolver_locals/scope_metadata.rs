use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_local_visibility_and_source() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Local, "a", true);
    symbols.set_import_source_for_test(Namespace::Local, "a", Some("std".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver local visibility/source mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has visibility public, expected private")),
        "expected resolver local visibility diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has source 'std', expected none")),
        "expected resolver local source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    0
}
"#,
    );
    let symbols_program = parse_program(
        r#"
main = () i32 {
    value = 1
    0
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver local should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra local symbol 'value'")),
        "expected extra resolver local diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_mutability_by_scope() {
    let program = parse_program(
        r#"
main = () i32 {
    value := 1
    {
        value := 2
        value
    }
    value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let inner_scope = symbols
        .symbols()
        .iter()
        .filter(|symbol| symbol.namespace == Namespace::Local && symbol.name == "value")
        .map(|symbol| symbol.scope_id)
        .max()
        .expect("inner value local");
    symbols.set_local_mutability_in_scope_for_test("value", inner_scope, Some(true));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver scoped local mutability mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'value' has mutability mutable, expected immutable")),
        "expected scoped resolver local mutability diagnostic, got {err:?}"
    );
}
