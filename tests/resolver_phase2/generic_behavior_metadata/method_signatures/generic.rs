use super::*;

#[test]
fn resolver_records_generic_behavior_method_signatures() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Json")
            .expect("behavior symbol")
            .behavior_method_signatures
            .as_deref(),
        Some(
            &[(
                "encode".to_string(),
                vec!["Self".to_string()],
                "T".to_string()
            )][..]
        )
    );
}

#[test]
fn resolver_records_generic_behavior_function_type_method_signatures() {
    let program = parse_program(
        r#"
Mapper<T>: behavior {
    map: (Self, (T) T) (T) T
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Mapper")
            .expect("behavior symbol")
            .behavior_method_signatures
            .as_deref(),
        Some(
            &[(
                "map".to_string(),
                vec!["Self".to_string(), "(T) T".to_string()],
                "(T) T".to_string()
            )][..]
        )
    );
}
