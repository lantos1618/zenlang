use super::*;

#[test]
fn check_program_with_symbols_validates_stripped_resolver_import_sources() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Import, "io", None);
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("stripped resolver imports without sources should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver import symbol 'io' has source 'unknown', expected a module source"
        )),
        "expected stripped resolver import source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_stripped_resolver_import_visibility() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Import, "io", true);
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("stripped resolver import visibility should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver import symbol 'io' has visibility public, expected private")),
        "expected stripped resolver import visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_requires_stripped_resolver_import_modules() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Module, "std");
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("stripped resolver imports should require source module symbols");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing module symbol 'std'")),
        "expected stripped resolver import module diagnostic, got {err:?}"
    );
}
