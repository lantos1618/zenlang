use super::*;

#[test]
fn resolver_rejects_method_on_unknown_type() {
    let err = resolver_errors(
        r#"
Missing.label = () StaticString { "missing" }
"#,
        "method receiver type should be known",
    );

    assert_resolver_error_contains(&err, "unknown type symbol 'Missing'");
}

#[test]
fn resolver_records_method_signatures_as_value_symbols() {
    let table = resolved_symbols(
        r#"
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    self.value
}
"#,
    );

    let method = symbol(&table, Namespace::Value, "Box.get");

    assert_string_metadata(method.parameter_names.as_deref(), &["self"]);
    assert_type_metadata(method.parameter_types.as_deref(), &["Box<T>"]);
    assert_type_name(method.return_type.as_ref(), Some("T"));
    assert_string_metadata(method.type_parameter_names.as_deref(), &["T"]);
    assert_type_parameter_bound_metadata(method.type_parameter_bound_refs.as_deref(), &[]);
}

#[test]
fn resolver_records_method_function_type_signatures() {
    let table = resolved_symbols(
        r#"
Box<T>: {
    value: T
}

Box.map<T> = (self: Box<T>, callback: (T) T) (T) T {
    callback
}
"#,
    );

    let method = symbol(&table, Namespace::Value, "Box.map");

    assert_string_metadata(method.parameter_names.as_deref(), &["self", "callback"]);
    assert_type_metadata(method.parameter_types.as_deref(), &["Box<T>", "(T) T"]);
    assert_type_name(method.return_type.as_ref(), Some("(T) T"));
    assert_string_metadata(method.type_parameter_names.as_deref(), &["T"]);
    assert_type_parameter_bound_metadata(method.type_parameter_bound_refs.as_deref(), &[]);
}

#[test]
fn resolver_rejects_self_type_outside_method_or_behavior() {
    let err = resolver_errors(
        r#"
main = (value: Self) i32 { 0 }
"#,
        "Self should require a method or behavior context",
    );

    assert_resolver_error_contains(&err, "Self type is only valid");
}
