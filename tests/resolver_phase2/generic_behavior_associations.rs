use super::*;

#[test]
fn resolver_records_behavior_parent_names() {
    let program = parse_program(
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

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "PrettyJson")
            .expect("behavior symbol")
            .behavior_parent_names
            .as_deref(),
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

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let point = table.lookup(Namespace::Type, "Point").expect("Point type");

    assert_eq!(
        point.behavior_impl_names.as_deref(),
        Some(&["Json<StaticString>".to_string()][..])
    );
    assert_eq!(
        point.behavior_impl_refs.as_deref(),
        Some(
            &[zen::resolver::BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![zen::ast::AstType::Str],
            }][..]
        )
    );
    assert_eq!(
        point.behavior_required_names.as_deref(),
        Some(&["Json<StaticString>".to_string()][..])
    );
    assert_eq!(
        point.behavior_required_refs.as_deref(),
        Some(
            &[zen::resolver::BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![zen::ast::AstType::Str],
            }][..]
        )
    );
}

#[test]
fn resolver_gates_generated_behavior_derive_association() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}

Point: { x: i32 }

Point.derive(Json)
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("generated derive associations should stay gated");

    assert!(
        err.iter().any(|d| {
            d.message
                .contains("generated behavior association `Type.derive(...)` is gated")
        }),
        "expected generated behavior association gate, got {err:?}"
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
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "PrettyJson")
            .expect("behavior symbol")
            .behavior_parent_names
            .as_deref(),
        Some(&["Json<StaticString>".to_string()][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Behavior, "PrettyJson")
            .expect("behavior symbol")
            .behavior_parent_refs
            .as_deref(),
        Some(
            &[zen::resolver::BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![zen::ast::AstType::Str],
            }][..]
        )
    );
}

#[test]
fn resolver_accepts_behavior_parent_type_args_from_child_type_params() {
    let program = parse_program(
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

    let table = Resolver::new()
        .resolve_program(&program)
        .expect("generic behavior parent should accept child type parameter args");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Pretty")
            .expect("behavior symbol")
            .behavior_parent_refs
            .as_deref(),
        Some(
            &[zen::resolver::BehaviorRefMetadata {
                name: "Serializable".to_string(),
                type_args: vec![zen::ast::AstType::Named("T".to_string())],
            }][..]
        )
    );
}

#[test]
fn resolver_rejects_behavior_parent_type_args_outside_child_type_params() {
    let program = parse_program(
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
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown parent type arg should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'U'")),
        "expected unknown type parameter diagnostic, got {err:?}"
    );
}
