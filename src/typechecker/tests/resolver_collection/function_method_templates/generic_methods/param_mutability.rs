use super::*;

#[test]
fn collect_declarations_with_symbols_preserves_generic_method_template_param_mutability_by_position(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, mut value: T) T {
    value = value
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { params, .. } = &mut program.declarations[1] {
        params[1].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic method template");
    assert_eq!(template.params[1].name, "value");
    assert!(
        template.params[1].mutable,
        "resolver-restored method parameter name should preserve positional mutability"
    );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_generic_method_template_param_names_for_mutability(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.choose<T> = (self: Box, left: T, mut right: T) T {
    right = right
    right
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { params, .. } = &mut program.declarations[1] {
        params.swap(1, 2);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.choose")
        .expect("generic method template");
    assert_eq!(template.params[1].name, "left");
    assert_eq!(template.params[2].name, "right");
    assert!(
        template.params[1].mutable,
        "resolver-restored first non-self method parameter should keep first AST position mutability"
    );
    assert!(
        !template.params[2].mutable,
        "resolver-restored second non-self method parameter should keep second AST position mutability"
    );
}
