use super::*;

#[test]
fn resolve_primitive_types() {
    let tc = TypeChecker::new();
    assert_eq!(tc.resolve_type(&AstType::I32), Type::I32);
    assert_eq!(tc.resolve_type(&AstType::F64), Type::F64);
    assert_eq!(tc.resolve_type(&AstType::Bool), Type::Bool);
    assert_eq!(tc.resolve_type(&AstType::Void), Type::Void);
    assert_eq!(tc.resolve_type(&AstType::Str), Type::Str);
}

#[test]
fn resolve_pointer_types() {
    let tc = TypeChecker::new();
    assert_eq!(
        tc.resolve_type(&AstType::Ptr(Box::new(AstType::I32))),
        Type::Ptr(Box::new(Type::I32))
    );
}

#[test]
fn method_signature_key_helpers_share_receiver_parsing() {
    assert_eq!(method_signature_key("Point", "get"), "Point.get");
    assert_eq!(
        method_signature_key_parts("Point.get"),
        Some(("Point", "get"))
    );
    assert_eq!(method_signature_receiver_name("Point.get"), Some("Point"));
    assert_eq!(
        method_signature_method_name_for_receiver("Point.get", "Point"),
        Some("get")
    );
    assert_eq!(
        method_signature_method_name_for_receiver("Other.get", "Point"),
        None
    );
    assert!(is_method_signature_key("Point.get"));
    assert_eq!(method_signature_key_parts("plain"), None);
    assert_eq!(method_signature_receiver_name("plain"), None);
    assert!(!is_method_signature_key("plain"));
}

#[test]
fn resolver_symbol_lookup_helpers_share_definition_span_fallbacks() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { self.x }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let Declaration::Method { span, .. } = &program.declarations[1] else {
        panic!("expected method declaration");
    };
    let span = *span;

    assert_eq!(
        TypeChecker::resolver_symbol_name_for(&symbols, Namespace::Value, "Point.missing", span),
        "Point.get"
    );
    assert_eq!(
        TypeChecker::resolver_method_signature_name_for(
            &symbols,
            "Missing.missing",
            "Missing",
            span
        ),
        "Point.get"
    );
    assert_eq!(
        TypeChecker::resolver_method_signature_symbol_by_span(&symbols, span)
            .map(|symbol| symbol.name.as_str()),
        Some("Point.get")
    );
}
