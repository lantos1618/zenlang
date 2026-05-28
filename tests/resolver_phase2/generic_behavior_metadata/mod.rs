mod method_signatures;

use super::*;

#[test]
fn resolver_records_type_and_behavior_generic_parameter_names() {
    let table = resolved_symbols(
        r#"
Box<T>: { value: T }
Option<T>: Some(T), None
Serializable<T>: behavior {
    encode: (T) StaticString
}
"#,
    );

    let t_names = ["T".to_string()];
    for (namespace, name) in [
        (Namespace::Type, "Box"),
        (Namespace::Type, "Option"),
        (Namespace::Behavior, "Serializable"),
    ] {
        let symbol = symbol(&table, namespace, name);
        assert_eq!(symbol.type_parameter_names.as_deref(), Some(&t_names[..]));
    }
}

#[test]
fn resolver_rejects_duplicate_type_parameter_names() {
    let err = resolver_errors(
        r#"
Box<T, T>: { value: T }
Option<T, T>: Some(T), None
Serializable<T, T>: behavior {
    encode: (T) StaticString
}
identity<T, T> = (value: T) T { value }
"#,
        "duplicate type parameter names should fail in resolver",
    );

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
    let table = resolved_symbols(
        r#"
Json: behavior {
    encode: (Self) StaticString
}
Box<T: Json>: { value: T }
Option<T: Json>: Some(T), None
Serializable<T: Json>: behavior {
    encode: (T) StaticString
}
"#,
    );

    for (namespace, name) in [
        (Namespace::Type, "Box"),
        (Namespace::Type, "Option"),
        (Namespace::Behavior, "Serializable"),
    ] {
        assert_type_parameter_bound_metadata(
            symbol(&table, namespace, name)
                .type_parameter_bound_refs
                .as_deref(),
            &[("T", "Json")],
        );
    }
}

#[test]
fn resolver_records_generic_behavior_bounds_with_type_args() {
    let table = resolved_symbols(
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

    assert_type_parameter_bound_metadata(
        symbol(&table, Namespace::Type, "Box")
            .type_parameter_bound_refs
            .as_deref(),
        &[("T", "Json<T>")],
    );
    assert_type_parameter_bound_metadata(
        symbol(&table, Namespace::Value, "encode")
            .type_parameter_bound_refs
            .as_deref(),
        &[("T", "Json<T>")],
    );
    assert_eq!(
        symbol(&table, Namespace::Value, "encode")
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
    assert_type_parameter_bound_metadata(
        symbol(&table, Namespace::Behavior, "Serializable")
            .type_parameter_bound_refs
            .as_deref(),
        &[("T", "Json<T>")],
    );
}
