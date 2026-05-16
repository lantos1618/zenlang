use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_method_signature_for_type_refs() {
    let mut program = parse_program(
        r#"
Box: { value: i32 }
Box.get = (self: Box, value: i32) i32 { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method {
        params,
        return_type,
        ..
    } = &mut program.declarations[1]
    {
        params[1].ty = AstType::Named("Missing".to_string());
        *return_type = Some(AstType::Named("AlsoMissing".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored method signature metadata should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_method_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { self.x }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method { method_name, .. } = &mut program.declarations[1] {
        *method_name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.methods.contains_key("Point.missing"));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_method_target_and_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { self.x }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Method {
        type_name,
        method_name,
        ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        *method_name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.methods.contains_key("Missing.missing"));
}

#[test]
fn collect_declarations_with_symbols_clears_stale_method_signature_after_key_restore() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { self.x }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.get", None);
    if let Declaration::Method {
        type_name,
        method_name,
        params,
        return_type,
        ..
    } = &mut program.declarations[1]
    {
        *type_name = "Missing".to_string();
        *method_name = "missing".to_string();
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.methods.contains_key("Missing.missing"),
            "resolver-backed collection should clear the stale AST method signature key after resolver key restoration"
        );
    assert!(
            !tc.methods.contains_key("Point.get"),
            "resolver-backed collection should clear the restored method signature key when resolver signature metadata is incomplete"
        );
}
