use super::*;

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
