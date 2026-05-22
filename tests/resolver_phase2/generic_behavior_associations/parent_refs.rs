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
