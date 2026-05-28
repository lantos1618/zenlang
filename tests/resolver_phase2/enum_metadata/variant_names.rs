use super::*;

#[test]
fn resolver_records_enum_variant_names() {
    let table = resolved_symbols(
        r#"
Option: Some(i32), None
"#,
    );

    assert_string_metadata(
        symbol(&table, Namespace::Type, "Option")
            .variant_names
            .as_deref(),
        &["Some", "None"],
    );
}

#[test]
fn resolver_allows_same_variant_names_in_different_enums() {
    let table = resolved_symbols(
        r#"
Option:
    None,
    Some(i32)

Maybe:
    None,
    Some(bool)
"#,
    );

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
    let err = resolver_errors(
        r#"
Option:
    None,
    None
"#,
        "duplicate variant names in one enum should be rejected",
    );

    assert_resolver_error_contains(&err, "duplicate variant symbol 'None'");
}
