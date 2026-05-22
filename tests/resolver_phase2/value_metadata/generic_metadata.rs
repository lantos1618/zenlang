use super::*;

#[test]
fn resolver_records_value_symbol_generic_parameter_counts() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T { value }
Point: { x: i32 }
Point.wrap<T> = (self: Point, value: T) Point { self }
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
            .as_deref(),
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
            .as_deref(),
        Some(&["T".to_string()][..])
    );
}

#[test]
fn resolver_records_value_symbol_generic_bounds() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}
encode<T: Json> = (value: T) StaticString { "encoded" }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Value, "encode")
            .expect("function symbol")
            .type_parameter_bounds
            .as_deref(),
        Some(&[("T".to_string(), "Json".to_string())][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Value, "encode")
            .expect("function symbol")
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
