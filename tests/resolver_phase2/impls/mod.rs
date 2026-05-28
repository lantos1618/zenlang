use super::*;
mod non_behavior;

#[test]
fn resolver_records_behavior_impl_methods_as_value_symbols() {
    let table = resolved_symbols(
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

    let method = symbol(&table, Namespace::Value, "Point.stringify");

    assert_eq!(method.name, "Point.stringify");
    assert_string_metadata(method.parameter_names.as_deref(), &["value"]);
    assert_type_metadata(method.parameter_types.as_deref(), &["Point"]);
    assert_type_name(method.return_type.as_ref(), Some("StaticString"));
    assert_string_metadata(method.type_parameter_names.as_deref(), &[]);
    assert_type_parameter_bound_metadata(method.type_parameter_bound_refs.as_deref(), &[]);
}

#[test]
fn resolver_rejects_duplicate_behavior_impl_edges() {
    let err = resolver_errors(
        r#"
Marker: behavior { }

Point: { x: i32 }

Point.implements(Marker) { }

Point.implements(Marker) { }
"#,
        "duplicate behavior impl should fail in resolver",
    );

    assert_resolver_error_contains(&err, "duplicate behavior implementation `Marker`");
}

#[test]
fn resolver_rejects_duplicate_behavior_impl_without_method_symbol_followup() {
    let err = resolver_errors(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "again" }
}
"#,
        "duplicate behavior impl should fail in resolver",
    );

    assert_eq!(
        err.len(),
        1,
        "duplicate behavior impl should not emit duplicate method symbol followups: {err:?}"
    );
    assert_resolver_error_contains(
        &err,
        "duplicate behavior implementation `Json<StaticString>`",
    );
}

#[test]
fn resolver_records_behavior_impl_function_type_methods() {
    let table = resolved_symbols(
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

    let method = symbol(&table, Namespace::Value, "Point.map");

    assert_eq!(method.name, "Point.map");
    assert_string_metadata(method.parameter_names.as_deref(), &["value", "callback"]);
    assert_type_metadata(method.parameter_types.as_deref(), &["Point", "(i32) i32"]);
    assert_type_name(method.return_type.as_ref(), Some("(i32) i32"));
    assert_string_metadata(method.type_parameter_names.as_deref(), &[]);
    assert_type_parameter_bound_metadata(method.type_parameter_bound_refs.as_deref(), &[]);
}

#[test]
fn resolver_records_behavior_impl_method_body_locals() {
    let table = resolved_symbols(
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

    let value = scoped_symbol(&table, Namespace::Local, "value");
    let label = scoped_symbol(&table, Namespace::Local, "label");

    assert_ne!(value.id, label.id);
    assert_ne!(value.scope_id, label.scope_id);
    assert_eq!(value.is_mutable, Some(false));
    assert_eq!(label.is_mutable, Some(false));
}
