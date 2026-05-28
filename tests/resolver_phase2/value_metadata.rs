use super::*;

fn value_symbol<'a>(table: &'a SymbolTable, name: &str) -> &'a Symbol {
    table.lookup(Namespace::Value, name).expect("value symbol")
}

#[test]
fn resolver_records_value_symbol_parameter_types() {
    let table = resolved_symbols(
        r#"
add = (a: i32, b: f64) f64 { b }
Point: { x: i32 }
Point.shift = (self: Point, dx: i32) Point { self }
"#,
    );

    assert_type_metadata(
        value_symbol(&table, "add").parameter_types.as_deref(),
        &["i32", "f64"],
    );
    assert_type_metadata(
        value_symbol(&table, "Point.shift")
            .parameter_types
            .as_deref(),
        &["Point", "i32"],
    );
}

#[test]
fn resolver_records_value_symbol_parameter_names() {
    let table = resolved_symbols(
        r#"
add = (a: i32, b: f64) f64 { b }
Point: { x: i32 }
Point.shift = (self: Point, dx: i32) Point { self }
"#,
    );

    assert_string_metadata(
        value_symbol(&table, "add").parameter_names.as_deref(),
        &["a", "b"],
    );
    assert_string_metadata(
        value_symbol(&table, "Point.shift")
            .parameter_names
            .as_deref(),
        &["self", "dx"],
    );
}

#[test]
fn resolver_records_value_symbol_return_types() {
    let table = resolved_symbols(
        r#"
main = () i32 { 0 }
log = () { }
"#,
    );

    assert_type_name(
        value_symbol(&table, "main").return_type.as_ref(),
        Some("i32"),
    );
    assert_type_name(
        value_symbol(&table, "log").return_type.as_ref(),
        Some("void"),
    );
}

#[test]
fn resolver_records_value_symbol_function_type_metadata() {
    let table = resolved_symbols(
        r#"
apply = (callback: (i32) i32, value: i32) (i32) i32 { callback }
"#,
    );
    let apply = value_symbol(&table, "apply");

    assert_type_metadata(apply.parameter_types.as_deref(), &["(i32) i32", "i32"]);
    assert_type_name(apply.return_type.as_ref(), Some("(i32) i32"));
}

#[test]
fn resolver_records_value_symbol_generic_parameter_names() {
    let table = resolved_symbols(
        r#"
identity<T> = (value: T) T { value }
Point: { x: i32 }
Point.wrap<T> = (self: Point, value: T) Point { self }
"#,
    );

    assert_string_metadata(
        value_symbol(&table, "identity")
            .type_parameter_names
            .as_deref(),
        &["T"],
    );
    assert_string_metadata(
        value_symbol(&table, "Point.wrap")
            .type_parameter_names
            .as_deref(),
        &["T"],
    );
}

#[test]
fn resolver_records_value_symbol_generic_bounds() {
    let table = resolved_symbols(
        r#"
Json: behavior {
    encode: (Self) StaticString
}
encode<T: Json> = (value: T) StaticString { "encoded" }
"#,
    );

    assert_type_parameter_bound_metadata(
        value_symbol(&table, "encode")
            .type_parameter_bound_refs
            .as_deref(),
        &[("T", "Json")],
    );
    assert_eq!(
        value_symbol(&table, "encode")
            .type_parameter_bound_refs
            .as_deref(),
        Some(
            &[TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: Vec::new(),
            }][..]
        )
    );
}
