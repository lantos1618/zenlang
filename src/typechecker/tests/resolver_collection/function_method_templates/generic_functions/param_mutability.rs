use super::*;

#[test]
fn collect_declarations_with_symbols_preserves_generic_template_param_mutability_by_position() {
    let mut program = parse_program(
        r#"
keep<T> = (mut value: T) T {
    value = value
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { params, .. } = &mut program.declarations[0] {
        params[0].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc.generic_functions.get("keep").expect("generic template");
    assert_eq!(template.params[0].name, "value");
    assert!(
        template.params[0].mutable,
        "resolver-restored parameter name should preserve positional mutability"
    );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_generic_template_param_names_for_mutability() {
    let mut program = parse_program(
        r#"
choose<T> = (left: T, mut right: T) T {
    right = right
    right
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { params, .. } = &mut program.declarations[0] {
        params.swap(0, 1);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("choose")
        .expect("generic template");
    assert_eq!(template.params[0].name, "left");
    assert_eq!(template.params[1].name, "right");
    assert!(
        template.params[0].mutable,
        "resolver-restored first parameter should keep first AST position mutability"
    );
    assert!(
        !template.params[1].mutable,
        "resolver-restored second parameter should keep second AST position mutability"
    );
}
