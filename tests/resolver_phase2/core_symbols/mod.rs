use super::*;
mod method_signatures;

#[test]
fn resolver_assigns_symbol_ids_in_separate_namespaces() {
    let table = resolved_symbols(
        r#"
Point: { x: i32 }
Point = () i32 { 1 }
Color: Red, Blue
"#,
    );

    let point_type = symbol(&table, Namespace::Type, "Point");
    let point_value = symbol(&table, Namespace::Value, "Point");
    let red_variant = symbol(&table, Namespace::Variant, "Red");

    assert_ne!(point_type.id, point_value.id);
    assert_ne!(point_type.id, red_variant.id);
    assert_eq!(point_type.name, "Point");
    assert_eq!(point_value.name, "Point");
    assert_eq!(red_variant.name, "Red");
    assert!(point_type.definition_span.start < point_type.definition_span.end);
}

#[test]
fn resolver_rejects_duplicate_names_in_same_namespace() {
    let err = resolver_errors(
        r#"
Point: { x: i32 }
Point: { y: i32 }
"#,
        "duplicate type name should fail",
    );

    assert_resolver_error_contains(&err, "duplicate type symbol 'Point'");
}

#[test]
fn resolver_records_public_visibility_for_exported_declarations() {
    let table = resolved_symbols(
        r#"
PublicPoint: { x: i32 }
PrivatePoint: { x: i32 }
Json<T>: behavior { encode: (Self) T }
InternalJson: behavior { encode: (Self) i32 }
exported = () i32 { 1 }
internal = () i32 { 2 }
@export({ PublicPoint, Json, exported })
"#,
    );

    for (namespace, name, is_public) in [
        (Namespace::Type, "PublicPoint", true),
        (Namespace::Type, "PrivatePoint", false),
        (Namespace::Behavior, "Json", true),
        (Namespace::Behavior, "InternalJson", false),
        (Namespace::Value, "exported", true),
        (Namespace::Value, "internal", false),
    ] {
        assert_eq!(symbol(&table, namespace, name).is_public, is_public);
    }
}

#[test]
fn resolver_rejects_unknown_type_references_in_declarations() {
    let err = resolver_errors(
        r#"
Point: { next: MissingPoint }
distance = (point: Point) UnknownReturn { 0 }
"#,
        "unknown type references should fail",
    );

    assert_resolver_error_contains(&err, "unknown type symbol 'MissingPoint'");
    assert_resolver_error_contains(&err, "unknown type symbol 'UnknownReturn'");
}

#[test]
fn resolver_records_import_bindings_as_symbols() {
    let table = resolved_symbols(
        r#"
{ ExternalPoint, helper } = geometry
distance = (point: ExternalPoint) i32 { helper() }
"#,
    );

    let module = symbol(&table, Namespace::Module, "geometry");
    let imported_type = symbol(&table, Namespace::Import, "ExternalPoint");
    let imported_value = symbol(&table, Namespace::Import, "helper");

    assert_ne!(module.id, imported_type.id);
    assert_ne!(imported_type.id, imported_value.id);
    assert_eq!(imported_type.import_source.as_deref(), Some("geometry"));
    assert_eq!(imported_value.import_source.as_deref(), Some("geometry"));
}
