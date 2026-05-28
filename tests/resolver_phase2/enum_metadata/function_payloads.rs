use super::*;

#[test]
fn resolver_records_enum_function_type_payloads() {
    let table = resolved_symbols(
        r#"
Callback: Wrap((i32) i32), None
"#,
    );

    let wrap = symbol(&table, Namespace::Variant, "Wrap");
    assert_type_name(wrap.variant_payload_type.as_ref(), Some("(i32) i32"));
    assert_eq!(
        wrap.variant_payload_type.as_ref(),
        Some(&zen::ast::AstType::Function {
            params: vec![zen::ast::AstType::I32],
            ret: Box::new(zen::ast::AstType::I32),
        })
    );
}

#[test]
fn resolver_records_generic_enum_function_type_payloads() {
    let table = resolved_symbols(
        r#"
Callback<T>: Wrap((T) T), None
"#,
    );

    assert_type_name(
        symbol(&table, Namespace::Variant, "Wrap")
            .variant_payload_type
            .as_ref(),
        Some("(T) T"),
    );
}
