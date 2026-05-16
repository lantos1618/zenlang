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

#[test]
fn resolver_records_behavior_method_signatures() {
    let program = parse_program(
        r#"
Serializable: behavior {
    encode: (Self, i32) str
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
                    "str".to_string()
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
    encode: (Self) str
    encode: (Self, i32) str
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
    encode: (value: Self, value: Self) str
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
    stringify: (Self) str {
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

#[test]
fn resolver_records_behavior_parent_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
PrettyJson: behavior {
    pretty: (Self) str
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
fn resolver_records_behavior_impl_and_requires_names() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let point = table.lookup(Namespace::Type, "Point").expect("Point type");

    assert_eq!(
        point.behavior_impl_names.as_deref(),
        Some(&["Json<str>".to_string()][..])
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
        Some(&["Json<str>".to_string()][..])
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

#[test]
fn resolver_records_generic_behavior_parent_names() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "PrettyJson")
            .expect("behavior symbol")
            .behavior_parent_names
            .as_deref(),
        Some(&["Json<str>".to_string()][..])
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
