use super::*;

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
