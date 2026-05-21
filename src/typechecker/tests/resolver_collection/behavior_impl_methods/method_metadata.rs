use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_metadata_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32
}

Point.implements(Mapper) {
    map = (self: Point, callback: (i32) i32) (i32) i32 { callback }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            params[1].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl method metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_name_metadata_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { name, .. } = &mut methods[0] {
            *name = "missing".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl method name metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_parameter_names_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (value: Self) StaticString
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params[0].name = "stale".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.encode").expect("impl method info");
    assert_eq!(info.params[0].0, "value");
    assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method parameter names should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_impl_method_parameter_order_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (value: Self, input: i32) StaticString
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) StaticString { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params.swap(0, 1);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.map").expect("impl method info");
    assert_eq!(info.params[0].0, "value");
    assert_eq!(info.params[1].0, "input");
    assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
    assert_eq!(info.params[1].1, AstType::I32);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method parameter order should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}
