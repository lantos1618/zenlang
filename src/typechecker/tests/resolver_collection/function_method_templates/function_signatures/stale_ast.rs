use super::*;

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_function_signature() {
    let mut program = parse_program(
        r#"
main = (value: i32) i32 { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "main", None);
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.functions.contains_key("main"),
            "resolver-backed collection should not keep AST-only function metadata when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_function_signature_after_name_restore() {
    let mut program = parse_program(
        r#"
main = (value: i32) i32 { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "main", None);
    if let Declaration::Function {
        name,
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        *name = "missing".to_string();
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.functions.contains_key("missing"),
            "resolver-backed collection should clear the stale AST function signature key after resolver name restoration"
        );
    assert!(
            !tc.functions.contains_key("main"),
            "resolver-backed collection should clear the restored function signature key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_function_template() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "identity", None);
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should not keep AST-only generic function templates when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_generic_function_body_refs_when_signature_incomplete(
) {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T {
    same: T = value
    same
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "identity", None);
    if let Declaration::Function { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should remove generic template when resolver signature metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST generic body refs when resolver signature metadata is incomplete: {:?}",
            tc.diagnostics
        );
}
