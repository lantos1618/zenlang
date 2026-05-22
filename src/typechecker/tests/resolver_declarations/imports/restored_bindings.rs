use super::*;

#[test]
fn check_program_with_symbols_uses_resolver_import_bindings() {
    let mut program = parse_program(
        r#"
{ io } = std
main = () i32 {
    io.println("ok")
    0
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    program
        .declarations
        .retain(|decl| !matches!(decl, Declaration::Import { .. }));

    let mut tc = TypeChecker::new();
    tc.check_program_with_symbols(&program, &symbols)
        .expect("resolver import symbols should seed typechecker imports");

    assert!(tc.is_root_std_import("io"));
}
