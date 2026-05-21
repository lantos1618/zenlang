use super::super::*;

#[test]
fn resolver_aggregate_metadata_collection_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_metadata_collection.rs");
    let aggregates = read("src/typechecker/resolver_metadata_collection/aggregates.rs");

    assert!(
        root.lines().count() < 160,
        "resolver_metadata_collection.rs should stay focused on behavior method metadata restoration"
    );
    assert!(
        root.contains("include!(\"resolver_metadata_collection/aggregates.rs\");"),
        "resolver metadata collection should include focused aggregate metadata collection"
    );

    for helper in [
        "collect_resolver_struct_fields",
        "resolver_struct_field_metadata",
        "resolver_struct_fields_from_metadata",
        "collect_resolver_enum_variants",
        "resolver_enum_variant_name_metadata",
        "resolver_enum_variants_from_metadata",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "resolver metadata collection root should not own aggregate helper: {helper}"
        );
        assert!(
            aggregates.contains(&format!("fn {helper}")),
            "aggregate metadata collection should live in aggregates.rs: {helper}"
        );
    }
}
