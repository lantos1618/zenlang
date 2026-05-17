#[path = "generic_behavior_metadata/method_signatures.rs"]
mod method_signatures;

use super::*;

#[test]
fn resolver_records_type_and_behavior_generic_parameter_counts() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
Option<T>: Some(T), None
Serializable<T>: behavior {
    encode: (T) str
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Box")
            .expect("struct symbol")
            .type_parameter_count,
        Some(1)
    );
    assert_eq!(
        table
            .lookup(Namespace::Type, "Box")
            .expect("struct symbol")
            .type_parameter_names
            .as_deref(),
        Some(&["T".to_string()][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Type, "Option")
            .expect("enum symbol")
            .type_parameter_count,
        Some(1)
    );
    assert_eq!(
        table
            .lookup(Namespace::Type, "Option")
            .expect("enum symbol")
            .type_parameter_names
            .as_deref(),
        Some(&["T".to_string()][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Serializable")
            .expect("behavior symbol")
            .type_parameter_count,
        Some(1)
    );
    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Serializable")
            .expect("behavior symbol")
            .type_parameter_names
            .as_deref(),
        Some(&["T".to_string()][..])
    );
}

#[test]
fn resolver_rejects_duplicate_type_parameter_names() {
    let program = parse_program(
        r#"
Box<T, T>: { value: T }
Option<T, T>: Some(T), None
Serializable<T, T>: behavior {
    encode: (T) str
}
identity<T, T> = (value: T) T { value }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate type parameter names should fail in resolver");

    let duplicate_count = err
        .iter()
        .filter(|d| d.message.contains("duplicate type parameter `T`"))
        .count();
    assert_eq!(
        duplicate_count, 4,
        "expected duplicate type parameter diagnostics for struct, enum, behavior, and function, got {err:?}"
    );
}

#[test]
fn resolver_records_type_and_behavior_generic_bounds() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
Box<T: Json>: { value: T }
Option<T: Json>: Some(T), None
Serializable<T: Json>: behavior {
    encode: (T) str
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Box")
            .expect("struct symbol")
            .type_parameter_bounds
            .as_deref(),
        Some(&[("T".to_string(), "Json".to_string())][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Type, "Option")
            .expect("enum symbol")
            .type_parameter_bounds
            .as_deref(),
        Some(&[("T".to_string(), "Json".to_string())][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Serializable")
            .expect("behavior symbol")
            .type_parameter_bounds
            .as_deref(),
        Some(&[("T".to_string(), "Json".to_string())][..])
    );
}

#[test]
fn resolver_records_generic_behavior_bounds_with_type_args() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box<T: Json<T>>: { value: T }
encode<T: Json<T>> = (value: T) T { value }
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Box")
            .expect("struct symbol")
            .type_parameter_bounds
            .as_deref(),
        Some(&[("T".to_string(), "Json<T>".to_string())][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Value, "encode")
            .expect("function symbol")
            .type_parameter_bounds
            .as_deref(),
        Some(&[("T".to_string(), "Json<T>".to_string())][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Value, "encode")
            .expect("function symbol")
            .type_parameter_bound_refs
            .as_deref(),
        Some(
            &[TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: vec![zen::ast::AstType::Named("T".to_string())],
            }][..]
        )
    );
    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Serializable")
            .expect("behavior symbol")
            .type_parameter_bounds
            .as_deref(),
        Some(&[("T".to_string(), "Json<T>".to_string())][..])
    );
}
