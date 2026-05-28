use super::*;

#[test]
fn resolver_records_behavior_method_signatures() {
    let table = resolved_symbols(
        r#"
Serializable: behavior {
    encode: (Self, i32) StaticString
    reset: () void
}
"#,
    );

    assert_method_signature_metadata(
        symbol(&table, Namespace::Behavior, "Serializable")
            .behavior_method_types
            .as_deref(),
        &[
            ("encode", &["Self", "i32"], "StaticString"),
            ("reset", &[], "void"),
        ],
    );
}

#[test]
fn resolver_rejects_duplicate_behavior_method_names() {
    let err = resolver_errors(
        r#"
Serializable: behavior {
    encode: (Self) StaticString
    encode: (Self, i32) StaticString
}
"#,
        "duplicate behavior method names should fail in resolver",
    );

    assert_resolver_error_contains(&err, "duplicate behavior method `encode` in `Serializable`");
}

#[test]
fn resolver_rejects_duplicate_signature_parameter_names() {
    let err = resolver_errors(
        r#"
Json: behavior {
    encode: (value: Self, value: Self) StaticString
}
"#,
        "duplicate behavior method parameter names should fail in resolver",
    );

    assert_resolver_error_contains(&err, "duplicate parameter `value`");
}

#[test]
fn resolver_records_behavior_function_type_method_signatures() {
    let table = resolved_symbols(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );

    assert_method_signature_metadata(
        symbol(&table, Namespace::Behavior, "Mapper")
            .behavior_method_types
            .as_deref(),
        &[("map", &["Self", "(i32) i32"], "(i32) i32")],
    );
    assert_eq!(
        symbol(&table, Namespace::Behavior, "Mapper")
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
    let table = resolved_symbols(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );

    assert_method_signature_metadata(
        symbol(&table, Namespace::Behavior, "Json")
            .behavior_method_types
            .as_deref(),
        &[("encode", &["Self"], "T")],
    );
}

#[test]
fn resolver_records_generic_behavior_function_type_method_signatures() {
    let table = resolved_symbols(
        r#"
Mapper<T>: behavior {
    map: (Self, (T) T) (T) T
}
"#,
    );

    assert_method_signature_metadata(
        symbol(&table, Namespace::Behavior, "Mapper")
            .behavior_method_types
            .as_deref(),
        &[("map", &["Self", "(T) T"], "(T) T")],
    );
}

#[test]
fn resolver_records_behavior_default_method_body_locals() {
    let table = resolved_symbols(
        r#"
Json: behavior {
    stringify: (Self) StaticString {
        label = "json"
        label
    }
}
"#,
    );

    let label = scoped_symbol(&table, Namespace::Local, "label");

    assert_eq!(label.is_mutable, Some(false));
    assert!(label.scope_id > 0);
}
