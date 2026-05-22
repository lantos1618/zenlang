use super::*;

#[test]
fn resolver_records_value_symbol_parameter_counts() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
Point: { x: i32 }
Point.shift = (self: Point, dx: i32) Point { self }
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
add = (a: i32, b: f64) f64 { b }
Point: { x: i32 }
Point.shift = (self: Point, dx: i32) Point { self }
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
add = (a: i32, b: f64) f64 { b }
Point: { x: i32 }
Point.shift = (self: Point, dx: i32) Point { self }
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
main = () i32 { 0 }
log = () { }
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
apply = (callback: (i32) i32, value: i32) (i32) i32 { callback }
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
