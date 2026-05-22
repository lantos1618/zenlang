use super::*;

#[test]
fn resolver_records_behavior_default_method_body_locals() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) StaticString {
        label = "json"
        label
    }
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let label = table
        .lookup_scoped(Namespace::Local, "label")
        .expect("behavior default body local symbol");

    assert_eq!(label.is_mutable, Some(false));
    assert!(label.scope_id > 0);
}
