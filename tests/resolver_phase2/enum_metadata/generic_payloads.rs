use super::*;

#[test]
fn resolver_records_generic_enum_variant_payload_types() {
    let program = parse_program(
        r#"
Result<T, E>: Ok(T), Err(E)
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Variant, "Ok")
            .expect("Ok variant symbol")
            .variant_payload_type_name
            .as_deref(),
        Some("T")
    );
    assert_eq!(
        table
            .lookup(Namespace::Variant, "Err")
            .expect("Err variant symbol")
            .variant_payload_type_name
            .as_deref(),
        Some("E")
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
