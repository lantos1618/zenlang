use super::*;

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
