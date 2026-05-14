use zen::error::FileTable;
use zen::lexer;
use zen::parser;
use zen::resolver::{Namespace, Resolver};

fn parse_program(src: &str) -> zen::ast::Program {
    let mut files = FileTable::new();
    let file_id = files.add_file("test.zen".to_string(), src.to_string());
    let tokens = lexer::tokenize(src, file_id).expect("tokenize");
    parser::parse(tokens, file_id).expect("parse")
}

#[test]
fn resolver_assigns_symbol_ids_in_separate_namespaces() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Point = () i32 { return 1 }
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
pub exported = () i32 { return 1 }
internal = () i32 { return 2 }
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
distance = (point: Point) UnknownReturn { return 0 }
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
Missing.label = () str { return "missing" }
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
    return self.value
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
    return callback
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
main = (value: Self) i32 { return 0 }
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
distance = (point: ExternalPoint) i32 { return helper() }
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

#[test]
fn resolver_records_behavior_impl_methods_as_value_symbols() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str { return "point" }
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let method = table
        .lookup(Namespace::Value, "Point.stringify")
        .expect("impl method symbol");

    assert_eq!(method.name, "Point.stringify");
    assert_eq!(method.parameter_count, Some(1));
    assert_eq!(
        method.parameter_names.as_deref(),
        Some(&["value".to_string()][..])
    );
    assert_eq!(
        method.parameter_type_names.as_deref(),
        Some(&["Point".to_string()][..])
    );
    assert_eq!(method.return_type_name.as_deref(), Some("str"));
    assert_eq!(method.type_parameter_count, Some(0));
    assert_eq!(method.type_parameter_names.as_deref(), Some(&[][..]));
    assert_eq!(method.type_parameter_bounds.as_deref(), Some(&[][..]));
}

#[test]
fn resolver_records_behavior_impl_function_type_methods() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}

Point: { x: i32 }

Point.implements(Mapper) {
    map = (value: Point, callback: (i32) i32) (i32) i32 {
        return callback
    }
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let method = table
        .lookup(Namespace::Value, "Point.map")
        .expect("impl method symbol");

    assert_eq!(method.name, "Point.map");
    assert_eq!(method.parameter_count, Some(2));
    assert_eq!(
        method.parameter_names.as_deref(),
        Some(&["value".to_string(), "callback".to_string()][..])
    );
    assert_eq!(
        method.parameter_type_names.as_deref(),
        Some(&["Point".to_string(), "(i32) i32".to_string()][..])
    );
    assert_eq!(method.return_type_name.as_deref(), Some("(i32) i32"));
    assert_eq!(method.type_parameter_count, Some(0));
    assert_eq!(method.type_parameter_names.as_deref(), Some(&[][..]));
    assert_eq!(method.type_parameter_bounds.as_deref(), Some(&[][..]));
}

#[test]
fn resolver_records_behavior_impl_method_body_locals() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str {
        label = "point"
        return label
    }
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let value = table
        .lookup_scoped(Namespace::Local, "value")
        .expect("impl method parameter symbol");
    let label = table
        .lookup_scoped(Namespace::Local, "label")
        .expect("impl method body local symbol");

    assert_ne!(value.id, label.id);
    assert_ne!(value.scope_id, label.scope_id);
    assert_eq!(value.is_mutable, Some(false));
    assert_eq!(label.is_mutable, Some(false));
}

#[test]
fn resolver_accepts_behavior_requires_known_type_and_behavior() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
"#,
    );

    Resolver::new()
        .resolve_program(&program)
        .expect("known requires assertion should resolve");
}

#[test]
fn resolver_rejects_behavior_requires_unknown_symbols() {
    let program = parse_program("Missing.requires(Json)");

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown requires symbols should fail");
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown type diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown behavior symbol 'Json'")),
        "expected unknown behavior diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_accepts_behavior_extends_known_behaviors() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );

    Resolver::new()
        .resolve_program(&program)
        .expect("known behavior inheritance should resolve");
}

#[test]
fn resolver_rejects_behavior_extends_unknown_symbols() {
    let program = parse_program("PrettyJson.extends(Json)");

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown behavior inheritance symbols should fail");
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown behavior symbol 'PrettyJson'")),
        "expected unknown child behavior diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown behavior symbol 'Json'")),
        "expected unknown parent behavior diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_records_value_symbol_parameter_counts() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { return a + b }
Point: { x: i32 }
Point.shift = (self: Point, dx: i32) Point { return self }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Value, "add")
            .expect("function symbol")
            .parameter_count,
        Some(2)
    );
    assert_eq!(
        table
            .lookup(Namespace::Value, "Point.shift")
            .expect("method symbol")
            .parameter_count,
        Some(2)
    );
}

#[test]
fn resolver_records_value_symbol_parameter_types() {
    let program = parse_program(
        r#"
add = (a: i32, b: f64) f64 { return b }
Point: { x: i32 }
Point.shift = (self: Point, dx: i32) Point { return self }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Value, "add")
            .expect("function symbol")
            .parameter_type_names
            .as_deref(),
        Some(&["i32".to_string(), "f64".to_string()][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Value, "Point.shift")
            .expect("method symbol")
            .parameter_type_names
            .as_deref(),
        Some(&["Point".to_string(), "i32".to_string()][..])
    );
}

#[test]
fn resolver_records_value_symbol_parameter_names() {
    let program = parse_program(
        r#"
add = (a: i32, b: f64) f64 { return b }
Point: { x: i32 }
Point.shift = (self: Point, dx: i32) Point { return self }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Value, "add")
            .expect("function symbol")
            .parameter_names
            .as_deref(),
        Some(&["a".to_string(), "b".to_string()][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Value, "Point.shift")
            .expect("method symbol")
            .parameter_names
            .as_deref(),
        Some(&["self".to_string(), "dx".to_string()][..])
    );
}

#[test]
fn resolver_records_value_symbol_return_types() {
    let program = parse_program(
        r#"
main = () i32 { return 0 }
log = () { return }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Value, "main")
            .expect("main symbol")
            .return_type_name
            .as_deref(),
        Some("i32")
    );
    assert_eq!(
        table
            .lookup(Namespace::Value, "log")
            .expect("log symbol")
            .return_type_name
            .as_deref(),
        Some("void")
    );
}

#[test]
fn resolver_records_value_symbol_function_type_metadata() {
    let program = parse_program(
        r#"
apply = (callback: (i32) i32, value: i32) (i32) i32 { return callback }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let apply = table
        .lookup(Namespace::Value, "apply")
        .expect("function symbol");

    assert_eq!(
        apply.parameter_type_names.as_deref(),
        Some(&["(i32) i32".to_string(), "i32".to_string()][..])
    );
    assert_eq!(apply.return_type_name.as_deref(), Some("(i32) i32"));
}

#[test]
fn resolver_records_value_symbol_generic_parameter_counts() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T { return value }
Point: { x: i32 }
Point.wrap<T> = (self: Point, value: T) Point { return self }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Value, "identity")
            .expect("function symbol")
            .type_parameter_count,
        Some(1)
    );
    assert_eq!(
        table
            .lookup(Namespace::Value, "identity")
            .expect("function symbol")
            .type_parameter_names
            .as_ref()
            .map(Vec::as_slice),
        Some(&["T".to_string()][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Value, "Point.wrap")
            .expect("method symbol")
            .type_parameter_count,
        Some(1)
    );
    assert_eq!(
        table
            .lookup(Namespace::Value, "Point.wrap")
            .expect("method symbol")
            .type_parameter_names
            .as_ref()
            .map(Vec::as_slice),
        Some(&["T".to_string()][..])
    );
}

#[test]
fn resolver_records_value_symbol_generic_bounds() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
encode<T: Json> = (value: T) str { return "encoded" }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Value, "encode")
            .expect("function symbol")
            .type_parameter_bounds
            .as_ref()
            .map(Vec::as_slice),
        Some(&[("T".to_string(), "Json".to_string())][..])
    );
}

#[test]
fn resolver_records_type_and_behavior_generic_parameter_counts() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
Option<T>: Some(T), None
Serializable<T>: behavior {
    encode: (T) str
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Box")
            .expect("struct symbol")
            .type_parameter_count,
        Some(1)
    );
    assert_eq!(
        table
            .lookup(Namespace::Type, "Box")
            .expect("struct symbol")
            .type_parameter_names
            .as_ref()
            .map(Vec::as_slice),
        Some(&["T".to_string()][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Type, "Option")
            .expect("enum symbol")
            .type_parameter_count,
        Some(1)
    );
    assert_eq!(
        table
            .lookup(Namespace::Type, "Option")
            .expect("enum symbol")
            .type_parameter_names
            .as_ref()
            .map(Vec::as_slice),
        Some(&["T".to_string()][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Serializable")
            .expect("behavior symbol")
            .type_parameter_count,
        Some(1)
    );
    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Serializable")
            .expect("behavior symbol")
            .type_parameter_names
            .as_ref()
            .map(Vec::as_slice),
        Some(&["T".to_string()][..])
    );
}

#[test]
fn resolver_records_type_and_behavior_generic_bounds() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
Box<T: Json>: { value: T }
Option<T: Json>: Some(T), None
Serializable<T: Json>: behavior {
    encode: (T) str
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Box")
            .expect("struct symbol")
            .type_parameter_bounds
            .as_ref()
            .map(Vec::as_slice),
        Some(&[("T".to_string(), "Json".to_string())][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Type, "Option")
            .expect("enum symbol")
            .type_parameter_bounds
            .as_ref()
            .map(Vec::as_slice),
        Some(&[("T".to_string(), "Json".to_string())][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Serializable")
            .expect("behavior symbol")
            .type_parameter_bounds
            .as_ref()
            .map(Vec::as_slice),
        Some(&[("T".to_string(), "Json".to_string())][..])
    );
}

#[test]
fn resolver_records_generic_behavior_bounds_with_type_args() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box<T: Json<T>>: { value: T }
encode<T: Json<T>> = (value: T) T { return value }
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Box")
            .expect("struct symbol")
            .type_parameter_bounds
            .as_ref()
            .map(Vec::as_slice),
        Some(&[("T".to_string(), "Json<T>".to_string())][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Value, "encode")
            .expect("function symbol")
            .type_parameter_bounds
            .as_ref()
            .map(Vec::as_slice),
        Some(&[("T".to_string(), "Json<T>".to_string())][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Serializable")
            .expect("behavior symbol")
            .type_parameter_bounds
            .as_ref()
            .map(Vec::as_slice),
        Some(&[("T".to_string(), "Json<T>".to_string())][..])
    );
}

#[test]
fn resolver_records_behavior_method_signatures() {
    let program = parse_program(
        r#"
Serializable: behavior {
    encode: (Self, i32) str
    reset: () void
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Serializable")
            .expect("behavior symbol")
            .behavior_method_signatures
            .as_ref()
            .map(Vec::as_slice),
        Some(
            &[
                (
                    "encode".to_string(),
                    vec!["Self".to_string(), "i32".to_string()],
                    "str".to_string()
                ),
                ("reset".to_string(), vec![], "void".to_string())
            ][..]
        )
    );
}

#[test]
fn resolver_records_behavior_function_type_method_signatures() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Mapper")
            .expect("behavior symbol")
            .behavior_method_signatures
            .as_ref()
            .map(Vec::as_slice),
        Some(
            &[(
                "map".to_string(),
                vec!["Self".to_string(), "(i32) i32".to_string()],
                "(i32) i32".to_string()
            )][..]
        )
    );
}

#[test]
fn resolver_records_generic_behavior_method_signatures() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Json")
            .expect("behavior symbol")
            .behavior_method_signatures
            .as_ref()
            .map(Vec::as_slice),
        Some(
            &[(
                "encode".to_string(),
                vec!["Self".to_string()],
                "T".to_string()
            )][..]
        )
    );
}

#[test]
fn resolver_records_behavior_default_method_body_locals() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str {
        label = "json"
        return label
    }
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let label = table
        .lookup_scoped(Namespace::Local, "label")
        .expect("behavior default body local symbol");

    assert_eq!(label.is_mutable, Some(false));
    assert!(label.scope_id > 0);
}

#[test]
fn resolver_records_behavior_parent_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "PrettyJson")
            .expect("behavior symbol")
            .behavior_parent_names
            .as_ref()
            .map(Vec::as_slice),
        Some(&["Json".to_string()][..])
    );
}

#[test]
fn resolver_records_behavior_impl_and_requires_names() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let point = table.lookup(Namespace::Type, "Point").expect("Point type");

    assert_eq!(
        point.behavior_impl_names.as_ref().map(Vec::as_slice),
        Some(&["Json<str>".to_string()][..])
    );
    assert_eq!(
        point.behavior_required_names.as_ref().map(Vec::as_slice),
        Some(&["Json<str>".to_string()][..])
    );
}

#[test]
fn resolver_records_generic_behavior_parent_names() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "PrettyJson")
            .expect("behavior symbol")
            .behavior_parent_names
            .as_ref()
            .map(Vec::as_slice),
        Some(&["Json<str>".to_string()][..])
    );
}

#[test]
fn resolver_records_struct_field_counts() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }
Empty: { }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Point")
            .expect("Point symbol")
            .field_count,
        Some(2)
    );
    assert_eq!(
        table
            .lookup(Namespace::Type, "Empty")
            .expect("Empty symbol")
            .field_count,
        Some(0)
    );
}

#[test]
fn resolver_records_struct_field_types() {
    let program = parse_program(
        r#"
Point: { x: i32, y: f64 }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Point")
            .expect("Point symbol")
            .field_type_names
            .as_ref()
            .map(Vec::as_slice),
        Some(
            &[
                ("x".to_string(), "i32".to_string()),
                ("y".to_string(), "f64".to_string())
            ][..]
        )
    );
}

#[test]
fn resolver_records_struct_function_type_fields() {
    let program = parse_program(
        r#"
Pipeline: { callback: (i32) i32 }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Pipeline")
            .expect("Pipeline symbol")
            .field_type_names
            .as_ref()
            .map(Vec::as_slice),
        Some(&[("callback".to_string(), "(i32) i32".to_string())][..])
    );
}

#[test]
fn resolver_records_generic_struct_field_types() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Box")
            .expect("Box symbol")
            .field_type_names
            .as_ref()
            .map(Vec::as_slice),
        Some(&[("value".to_string(), "T".to_string())][..])
    );
}

#[test]
fn resolver_records_struct_field_default_locals() {
    let program = parse_program(
        r#"
Point: {
    x: i32 = {
        value = 1
        value
    }
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let value = table
        .lookup_scoped(Namespace::Local, "value")
        .expect("struct field default local symbol");

    assert_eq!(value.is_mutable, Some(false));
    assert!(value.scope_id > 0);
}

#[test]
fn resolver_rejects_duplicate_struct_literal_fields() {
    let program = parse_program(
        r#"
Point: { x: i32 }

main = () i32 {
    point = Point { x: 1, x: 2 }
    return 0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate struct literal field should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate field `x` for struct `Point`")),
        "expected duplicate struct literal field diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_unknown_struct_literal_fields() {
    let program = parse_program(
        r#"
Point: { x: i32 }

main = () i32 {
    point = Point { x: 1, y: 2 }
    return 0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown struct literal field should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown field `y` for struct `Point`")),
        "expected unknown struct literal field diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_missing_struct_literal_fields() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }

main = () i32 {
    point = Point { x: 1 }
    return 0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("missing struct literal field should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("missing field `y` for struct `Point`")),
        "expected missing struct literal field diagnostic, got {err:?}"
    );
}

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
            .as_ref()
            .map(Vec::as_slice),
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
}

#[test]
fn resolver_rejects_unknown_unqualified_function_calls() {
    let program = parse_program(
        r#"
known = () i32 { return 1 }
main = () i32 { return missing() }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown function call should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown value symbol 'missing'")),
        "expected missing function diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_records_parameter_and_local_symbols() {
    let program = parse_program(
        r#"
main = (mut input: i32) i32 {
    value ::= input
    frozen = value
    return frozen
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let input = table
        .lookup_scoped(Namespace::Local, "input")
        .expect("parameter symbol");
    let value = table
        .lookup_scoped(Namespace::Local, "value")
        .expect("local symbol");
    let frozen = table
        .lookup_scoped(Namespace::Local, "frozen")
        .expect("immutable local symbol");

    assert_ne!(input.id, value.id);
    assert_ne!(input.scope_id, value.scope_id);
    assert_eq!(input.is_mutable, Some(true));
    assert_eq!(value.is_mutable, Some(true));
    assert_eq!(frozen.is_mutable, Some(false));
}

#[test]
fn resolver_records_top_level_expr_locals() {
    let program = parse_program(
        r#"
value := 1
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let value = table
        .lookup_scoped(Namespace::Local, "value")
        .expect("top-level expr local symbol");

    assert_eq!(value.is_mutable, Some(false));
    assert!(value.scope_id > 0);
}

#[test]
fn resolver_rejects_unknown_enum_variant_expressions() {
    let program = parse_program(
        r#"
Status: Ok, Err

main = () i32 {
    value = Status.Pending
    return 0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown enum variant expression should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("enum `Status` has no variant `Pending`")),
        "expected unknown enum variant diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_missing_enum_variant_payload_expressions() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

main = () i32 {
    value = Maybe.Some
    return 0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("missing enum variant payload expression should fail in resolver");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("enum variant `Maybe.Some` requires a payload")),
        "expected missing enum variant payload diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_unexpected_enum_variant_payload_expressions() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

main = () i32 {
    value = Maybe.None(1)
    return 0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unexpected enum variant payload expression should fail in resolver");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("enum variant `Maybe.None` does not accept a payload")),
        "expected unexpected enum variant payload diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_records_closure_locals() {
    let program = parse_program(
        r#"
main = () i32 {
    mapper = (input: i32) i32 {
        inner = input
        inner
    }
    return 0
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let mapper = table
        .lookup_scoped(Namespace::Local, "mapper")
        .expect("closure binding local symbol");
    let input = table
        .lookup_scoped(Namespace::Local, "input")
        .expect("closure parameter local symbol");
    let inner = table
        .lookup_scoped(Namespace::Local, "inner")
        .expect("closure body local symbol");

    assert_ne!(mapper.scope_id, input.scope_id);
    assert_ne!(input.scope_id, inner.scope_id);
    assert!(inner.scope_id > input.scope_id);
    assert_eq!(mapper.is_mutable, Some(false));
    assert_eq!(input.is_mutable, Some(false));
    assert_eq!(inner.is_mutable, Some(false));
}

#[test]
fn resolver_records_pattern_locals() {
    let program = parse_program(
        r#"
Option:
    None,
    Some(i32)

main = (value: Option) i32 {
    return value ?
        | Some(inner) { inner }
        | None { 0 }
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let value = table
        .lookup_scoped(Namespace::Local, "value")
        .expect("parameter local symbol");
    let inner = table
        .lookup_scoped(Namespace::Local, "inner")
        .expect("pattern local symbol");

    assert_ne!(value.scope_id, inner.scope_id);
    assert_eq!(inner.is_mutable, Some(false));
    assert!(inner.scope_id > value.scope_id);
}

#[test]
fn resolver_records_same_name_locals_in_distinct_scopes() {
    let program = parse_program(
        r#"
main = () i32 {
    value := 1
    {
        value := 2
        value
    }
    return value
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let values: Vec<_> = table
        .symbols()
        .iter()
        .filter(|symbol| symbol.namespace == Namespace::Local && symbol.name == "value")
        .collect();

    assert_eq!(values.len(), 2);
    assert_ne!(values[0].id, values[1].id);
    assert_ne!(values[0].scope_id, values[1].scope_id);
    assert!(values.iter().all(|symbol| symbol.is_mutable == Some(false)));
}

#[test]
fn resolver_rejects_duplicate_bindings_in_same_scope() {
    let program = parse_program(
        r#"
main = (input: i32, input: i32) i32 {
    value = 1
    value = 2
    return value
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate locals should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate local symbol 'input'")),
        "expected duplicate parameter diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate local symbol 'value'")),
        "expected duplicate local diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_unknown_local_identifier_references() {
    let program = parse_program(
        r#"
main = () i32 {
    return missing_local
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown local identifier should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown value symbol 'missing_local'")),
        "expected missing local diagnostic, got {err:?}"
    );
}
