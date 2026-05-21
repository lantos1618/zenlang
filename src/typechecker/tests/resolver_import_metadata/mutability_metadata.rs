use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_import_and_module_absent_mutability() {
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
    symbols.set_mutability_for_test(Namespace::Import, "io", Some(true));
    symbols.set_mutability_for_test(Namespace::Module, "std", Some(false));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import/module mutability metadata should fail");

    for expected in [
        "resolver import symbol 'io' has mutability metadata, expected none",
        "resolver module symbol 'std' has mutability metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver import/module mutability diagnostic `{expected}`, got {err:?}"
        );
    }
}
