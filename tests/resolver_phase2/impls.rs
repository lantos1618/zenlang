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
fn resolver_rejects_duplicate_behavior_impl_edges() {
    let program = parse_program(
        r#"
Marker: behavior { }

Point: { x: i32 }

Point.implements(Marker) { }

Point.implements(Marker) { }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate behavior impl should fail in resolver");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("duplicate behavior implementation `Marker`")),
        "expected duplicate behavior implementation diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_duplicate_behavior_impl_without_method_symbol_followup() {
    let program = parse_program(
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
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate behavior impl should fail in resolver");

    assert_eq!(
        err.len(),
        1,
        "duplicate behavior impl should not emit duplicate method symbol followups: {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("duplicate behavior implementation `Json<StaticString>`")),
        "expected duplicate behavior implementation diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_accepts_non_behavior_impl_blocks_as_method_symbols() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
    );

    let symbols = Resolver::new()
        .resolve_program(&program)
        .expect("non-behavior impl blocks should resolve");

    let get = symbols
        .lookup(Namespace::Value, "Point.get")
        .expect("impl method symbol");
    assert_eq!(get.parameter_count, Some(1));
    assert_eq!(
        get.parameter_type_names.as_deref(),
        Some(&["Point".to_string()][..])
    );
}

#[test]
fn resolver_rejects_duplicate_non_behavior_impl_method_names() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
    get = (self: Point) i32 { self.x }
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate non-behavior impl methods should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate value symbol 'Point.get'")),
        "expected duplicate impl method symbol diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_non_behavior_impl_method_colliding_with_top_level_method() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Point.get = (self: Point) i32 { self.x }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("impl method colliding with top-level method should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate value symbol 'Point.get'")),
        "expected duplicate method symbol diagnostic, got {err:?}"
    );
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
