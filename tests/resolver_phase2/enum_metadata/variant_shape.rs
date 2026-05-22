use super::*;

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
