use super::*;

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
