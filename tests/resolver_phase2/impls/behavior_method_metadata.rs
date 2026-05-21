use super::*;

#[test]
fn resolver_records_behavior_impl_methods_as_value_symbols() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) StaticString
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) StaticString { "point" }
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
    assert_eq!(method.return_type_name.as_deref(), Some("StaticString"));
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
        callback
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
    stringify: (Self) StaticString
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) StaticString {
        label = "point"
        label
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
