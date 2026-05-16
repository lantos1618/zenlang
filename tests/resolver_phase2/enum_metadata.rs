use super::*;

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
fn resolver_records_enum_variant_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Option")
            .expect("Option type symbol")
            .variant_names
            .as_deref(),
        Some(&["Some".to_string(), "None".to_string()][..])
    );
}

#[test]
fn resolver_allows_same_variant_names_in_different_enums() {
    let program = parse_program(
        r#"
Option:
    None,
    Some(i32)

Maybe:
    None,
    Some(bool)
"#,
    );

    let table = Resolver::new()
        .resolve_program(&program)
        .expect("variant names should be scoped to their owner enum");

    assert_eq!(
        table
            .symbols()
            .iter()
            .filter(|symbol| symbol.namespace == Namespace::Variant && symbol.name == "None")
            .count(),
        2
    );
    assert_eq!(
        table
            .symbols()
            .iter()
            .filter(|symbol| symbol.namespace == Namespace::Variant && symbol.name == "Some")
            .count(),
        2
    );
}

#[test]
fn resolver_rejects_duplicate_variant_names_in_same_enum() {
    let program = parse_program(
        r#"
Option:
    None,
    None
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate variant names in one enum should be rejected");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate variant symbol 'None'")),
        "expected duplicate variant diagnostic, got {err:?}"
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
