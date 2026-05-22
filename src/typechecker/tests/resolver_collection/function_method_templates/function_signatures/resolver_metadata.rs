use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_type_metadata() {
    let mut program = parse_program(
        r#"
apply = (callback: (i32) i32) (i32) i32 {
    callback
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::I32;
        *return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.functions.get("apply").expect("function info");
    assert_eq!(
        info.params[0].1,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
    assert_eq!(
        info.return_type,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_signature_for_type_refs() {
    let mut program = parse_program(
        r#"
main = (value: i32) i32 { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        params[0].ty = AstType::Named("Missing".to_string());
        *return_type = Some(AstType::Named("AlsoMissing".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored function signature metadata should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}
