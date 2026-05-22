use super::*;

#[path = "core_symbols/method_signatures.rs"]
mod method_signatures;

#[test]
fn resolver_assigns_symbol_ids_in_separate_namespaces() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Point = () i32 { 1 }
Color: Red, Blue
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let point_type = table.lookup(Namespace::Type, "Point").expect("Point type");
    let point_value = table
        .lookup(Namespace::Value, "Point")
        .expect("Point function");
    let red_variant = table
        .lookup(Namespace::Variant, "Red")
        .expect("Red variant");

    assert_ne!(point_type.id, point_value.id);
    assert_ne!(point_type.id, red_variant.id);
    assert_eq!(point_type.name, "Point");
    assert_eq!(point_value.name, "Point");
    assert_eq!(red_variant.name, "Red");
    assert!(point_type.definition_span.start < point_type.definition_span.end);
}

#[test]
fn resolver_rejects_duplicate_names_in_same_namespace() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Point: { y: i32 }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate type name should fail");
    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate type symbol 'Point'")),
        "expected duplicate type diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_records_public_visibility_for_exported_declarations() {
    let program = parse_program(
        r#"
pub PublicPoint: { x: i32 }
PrivatePoint: { x: i32 }
pub Json<T>: behavior { encode: (Self) T }
InternalJson: behavior { encode: (Self) i32 }
pub exported = () i32 { 1 }
internal = () i32 { 2 }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert!(
        table
            .lookup(Namespace::Type, "PublicPoint")
            .expect("public type")
            .is_public
    );
    assert!(
        !table
            .lookup(Namespace::Type, "PrivatePoint")
            .expect("private type")
            .is_public
    );
    assert!(
        table
            .lookup(Namespace::Behavior, "Json")
            .expect("public behavior")
            .is_public
    );
    assert!(
        !table
            .lookup(Namespace::Behavior, "InternalJson")
            .expect("private behavior")
            .is_public
    );
    assert!(
        table
            .lookup(Namespace::Value, "exported")
            .expect("public function")
            .is_public
    );
    assert!(
        !table
            .lookup(Namespace::Value, "internal")
            .expect("private function")
            .is_public
    );
}

#[test]
fn resolver_rejects_unknown_type_references_in_declarations() {
    let program = parse_program(
        r#"
Point: { next: MissingPoint }
distance = (point: Point) UnknownReturn { 0 }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown type references should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'MissingPoint'")),
        "expected missing field type diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'UnknownReturn'")),
        "expected missing return type diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_records_import_bindings_as_symbols() {
    let program = parse_program(
        r#"
{ ExternalPoint, helper } = geometry
distance = (point: ExternalPoint) i32 { helper() }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let module = table
        .lookup(Namespace::Module, "geometry")
        .expect("module symbol");
    let imported_type = table
        .lookup(Namespace::Import, "ExternalPoint")
        .expect("imported type binding");
    let imported_value = table
        .lookup(Namespace::Import, "helper")
        .expect("imported value binding");

    assert_ne!(module.id, imported_type.id);
    assert_ne!(imported_type.id, imported_value.id);
    assert_eq!(imported_type.import_source.as_deref(), Some("geometry"));
    assert_eq!(imported_value.import_source.as_deref(), Some("geometry"));
}
