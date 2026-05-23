use super::*;

#[test]
fn resolver_records_enum_function_type_payloads() {
    let program = parse_program(
        r#"
Callback: Wrap((i32) i32), None
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Variant, "Wrap")
            .expect("Wrap variant symbol")
            .variant_payload_type_name
            .as_deref(),
        Some("(i32) i32")
    );
    assert_eq!(
        table
            .lookup(Namespace::Variant, "Wrap")
            .expect("Wrap variant symbol")
            .variant_payload_type
            .as_ref(),
        Some(&zen::ast::AstType::Function {
            params: vec![zen::ast::AstType::I32],
            ret: Box::new(zen::ast::AstType::I32),
        })
    );
}

#[test]
fn resolver_records_generic_enum_function_type_payloads() {
    let program = parse_program(
        r#"
Callback<T>: Wrap((T) T), None
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Variant, "Wrap")
            .expect("Wrap variant symbol")
            .variant_payload_type_name
            .as_deref(),
        Some("(T) T")
    );
}
