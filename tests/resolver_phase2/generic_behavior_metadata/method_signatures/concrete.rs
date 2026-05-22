use super::*;

#[test]
fn resolver_records_behavior_method_signatures() {
    let program = parse_program(
        r#"
Serializable: behavior {
    encode: (Self, i32) StaticString
    reset: () void
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Serializable")
            .expect("behavior symbol")
            .behavior_method_signatures
            .as_deref(),
        Some(
            &[
                (
                    "encode".to_string(),
                    vec!["Self".to_string(), "i32".to_string()],
                    "StaticString".to_string()
                ),
                ("reset".to_string(), vec![], "void".to_string())
            ][..]
        )
    );
}
