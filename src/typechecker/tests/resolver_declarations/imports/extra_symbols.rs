use super::*;

#[test]
fn check_program_with_symbols_rejects_extra_resolver_imports_when_ast_imports_are_present() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let symbols_program = parse_program(
        r#"
{ io, math } = std
main = () i32 { 0 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver imports should fail when AST imports are present");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra import symbol 'math'")),
        "expected extra resolver import diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_modules_when_ast_imports_are_present() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let symbols_program = parse_program(
        r#"
{ io } = std
{ helper } = other
main = () i32 { 0 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&symbols_program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver modules should fail when AST imports are present");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table has extra module symbol 'other'")),
        "expected extra resolver module diagnostic, got {err:?}"
    );
}
