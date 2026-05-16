use super::*;

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
fn resolver_rejects_method_on_unknown_type() {
    let program = parse_program(
        r#"
Missing.label = () str { "missing" }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("method receiver type should be known");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown method receiver type diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_records_method_signatures_as_value_symbols() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    self.value
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let method = table
        .lookup(Namespace::Value, "Box.get")
        .expect("method symbol");

    assert_eq!(method.parameter_count, Some(1));
    assert_eq!(
        method.parameter_names.as_deref(),
        Some(&["self".to_string()][..])
    );
    assert_eq!(
        method.parameter_type_names.as_deref(),
        Some(&["Box<T>".to_string()][..])
    );
    assert_eq!(method.return_type_name.as_deref(), Some("T"));
    assert_eq!(method.type_parameter_count, Some(1));
    assert_eq!(
        method.type_parameter_names.as_deref(),
        Some(&["T".to_string()][..])
    );
    assert_eq!(method.type_parameter_bounds.as_deref(), Some(&[][..]));
}

#[test]
fn resolver_records_method_function_type_signatures() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

Box.map<T> = (self: Box<T>, callback: (T) T) (T) T {
    callback
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let method = table
        .lookup(Namespace::Value, "Box.map")
        .expect("method symbol");

    assert_eq!(method.parameter_count, Some(2));
    assert_eq!(
        method.parameter_names.as_deref(),
        Some(&["self".to_string(), "callback".to_string()][..])
    );
    assert_eq!(
        method.parameter_type_names.as_deref(),
        Some(&["Box<T>".to_string(), "(T) T".to_string()][..])
    );
    assert_eq!(method.return_type_name.as_deref(), Some("(T) T"));
    assert_eq!(method.type_parameter_count, Some(1));
    assert_eq!(
        method.type_parameter_names.as_deref(),
        Some(&["T".to_string()][..])
    );
    assert_eq!(method.type_parameter_bounds.as_deref(), Some(&[][..]));
}

#[test]
fn resolver_rejects_self_type_outside_method_or_behavior() {
    let program = parse_program(
        r#"
main = (value: Self) i32 { 0 }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("Self should require a method or behavior context");

    assert!(
        err.iter()
            .any(|d| d.message.contains("Self type is only valid")),
        "expected invalid Self type diagnostic, got {err:?}"
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
