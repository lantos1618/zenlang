use super::*;

#[test]
fn resolver_records_behavior_parent_refs() {
    let table = resolved_symbols(
        r#"
Json: behavior {
    encode: (Self) StaticString
}
PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)
"#,
    );

    assert_behavior_refs(
        symbol(&table, Namespace::Behavior, "PrettyJson")
            .behavior_parent_refs
            .as_deref(),
        &[("Json", Vec::new())],
    );
}

#[test]
fn resolver_records_behavior_impl_and_requires_refs() {
    let table = resolved_symbols(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );

    let point = symbol(&table, Namespace::Type, "Point");

    assert_behavior_refs(
        point.behavior_impl_refs.as_deref(),
        &[("Json", vec![zen::ast::AstType::Str])],
    );
    assert_behavior_refs(
        point.behavior_required_refs.as_deref(),
        &[("Json", vec![zen::ast::AstType::Str])],
    );
}

#[test]
fn resolver_gates_generated_behavior_derive_association() {
    let err = resolver_errors(
        r#"
Json: behavior {
    encode: (Self) StaticString
}

Point: { x: i32 }

Point.derive(Json)
"#,
        "generated derive associations should stay gated",
    );

    assert_resolver_error_contains(
        &err,
        "generated behavior association `Type.derive(...)` is gated",
    );
}

#[test]
fn resolver_records_generic_behavior_parent_refs() {
    let table = resolved_symbols(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)
"#,
    );

    let pretty_json = symbol(&table, Namespace::Behavior, "PrettyJson");
    assert_behavior_refs(
        pretty_json.behavior_parent_refs.as_deref(),
        &[("Json", vec![zen::ast::AstType::Str])],
    );
}

#[test]
fn resolver_accepts_behavior_parent_type_args_from_child_type_params() {
    let table = resolved_symbols(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
    );

    assert_behavior_refs(
        symbol(&table, Namespace::Behavior, "Pretty")
            .behavior_parent_refs
            .as_deref(),
        &[(
            "Serializable",
            vec![zen::ast::AstType::Named("T".to_string())],
        )],
    );
}

#[test]
fn resolver_rejects_behavior_parent_type_args_outside_child_type_params() {
    let err = resolver_errors(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<U>)
"#,
        "unknown parent type arg should fail in resolver",
    );

    assert_resolver_error_contains(&err, "unknown type symbol 'U'");
}
