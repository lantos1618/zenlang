use super::*;
mod function_payloads;
mod variant_names;

#[test]
fn resolver_records_enum_variant_owner_names() {
    let table = resolved_symbols(
        r#"
Option: Some(i32), None
"#,
    );

    assert_eq!(
        symbol(&table, Namespace::Variant, "Some")
            .variant_owner_name
            .as_deref(),
        Some("Option")
    );
    assert_eq!(
        symbol(&table, Namespace::Variant, "None")
            .variant_owner_name
            .as_deref(),
        Some("Option")
    );
}

#[test]
fn resolver_records_enum_variant_payload_types() {
    let table = resolved_symbols(
        r#"
Option: Some(i32), None
"#,
    );

    assert_type_name(
        symbol(&table, Namespace::Variant, "Some")
            .variant_payload_type
            .as_ref(),
        Some("i32"),
    );
    assert_type_name(
        symbol(&table, Namespace::Variant, "None")
            .variant_payload_type
            .as_ref(),
        None,
    );
}

#[test]
fn resolver_records_generic_enum_variant_payload_types() {
    let table = resolved_symbols(
        r#"
Result<T, E>: Ok(T), Err(E)
"#,
    );

    assert_type_name(
        symbol(&table, Namespace::Variant, "Ok")
            .variant_payload_type
            .as_ref(),
        Some("T"),
    );
    assert_type_name(
        symbol(&table, Namespace::Variant, "Err")
            .variant_payload_type
            .as_ref(),
        Some("E"),
    );
}
