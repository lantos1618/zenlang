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

#[test]
fn resolver_rejects_duplicate_behavior_method_names() {
    let program = parse_program(
        r#"
Serializable: behavior {
    encode: (Self) StaticString
    encode: (Self, i32) StaticString
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate behavior method names should fail in resolver");

    assert!(
        err.iter().any(|d| {
            d.message
                .contains("duplicate behavior method `encode` in `Serializable`")
        }),
        "expected duplicate behavior method diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_duplicate_signature_parameter_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (value: Self, value: Self) StaticString
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate behavior method parameter names should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate parameter `value`")),
        "expected duplicate behavior method parameter diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_records_behavior_function_type_method_signatures() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
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
                vec!["Self".to_string(), "(i32) i32".to_string()],
                "(i32) i32".to_string()
            )][..]
        )
    );
    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Mapper")
            .expect("behavior symbol")
            .behavior_method_types
            .as_deref(),
        Some(
            &[zen::resolver::BehaviorMethodTypeMetadata {
                name: "map".to_string(),
                parameter_names: vec!["__arg0".to_string(), "__arg1".to_string()],
                parameter_types: vec![
                    zen::ast::AstType::SelfType,
                    zen::ast::AstType::Function {
                        params: vec![zen::ast::AstType::I32],
                        ret: Box::new(zen::ast::AstType::I32),
                    },
                ],
                return_type: zen::ast::AstType::Function {
                    params: vec![zen::ast::AstType::I32],
                    ret: Box::new(zen::ast::AstType::I32),
                },
            }][..]
        )
    );
}

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
