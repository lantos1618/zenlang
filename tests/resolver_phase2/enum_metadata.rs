use super::*;

#[path = "enum_metadata/function_payloads.rs"]
mod function_payloads;
#[path = "enum_metadata/variant_names.rs"]
mod variant_names;

#[test]
fn resolver_records_enum_variant_payload_counts() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Variant, "Some")
            .expect("Some variant symbol")
            .variant_payload_count,
        Some(1)
    );
    assert_eq!(
        table
            .lookup(Namespace::Variant, "None")
            .expect("None variant symbol")
            .variant_payload_count,
        Some(0)
    );
}

#[test]
fn resolver_records_enum_variant_owner_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Variant, "Some")
            .expect("Some variant symbol")
            .variant_owner_name
            .as_deref(),
        Some("Option")
    );
    assert_eq!(
        table
            .lookup(Namespace::Variant, "None")
            .expect("None variant symbol")
            .variant_owner_name
            .as_deref(),
        Some("Option")
    );
}

#[test]
fn resolver_records_enum_variant_payload_types() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Variant, "Some")
            .expect("Some variant symbol")
            .variant_payload_type_name
            .as_deref(),
        Some("i32")
    );
    assert_eq!(
        table
            .lookup(Namespace::Variant, "None")
            .expect("None variant symbol")
            .variant_payload_type_name
            .as_deref(),
        None
    );
}

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
