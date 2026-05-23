use super::*;

mod helpers;

#[test]
fn resolver_callable_signature_metadata_requires_complete_signature() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let signature = TypeChecker::resolver_callable_signature_metadata(
        symbols
            .lookup(Namespace::Value, "identity")
            .expect("identity symbol"),
    )
    .expect("complete resolver signature");

    assert_eq!(signature.parameter_names, ["value"]);
    assert_eq!(signature.parameter_types, [AstType::Named("T".to_string())]);
    assert_eq!(signature.return_type, &AstType::Named("T".to_string()));

    symbols.set_parameter_types_for_test(Namespace::Value, "identity", None);
    assert!(TypeChecker::resolver_callable_signature_metadata(
        symbols
            .lookup(Namespace::Value, "identity")
            .expect("identity symbol")
    )
    .is_none());
}

#[test]
fn resolver_struct_field_metadata_requires_field_types() {
    let program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let fields = TypeChecker::resolver_struct_field_metadata(
        symbols
            .lookup(Namespace::Type, "Point")
            .expect("Point symbol"),
    )
    .expect("complete resolver fields");

    assert_eq!(fields, [("x".to_string(), AstType::I32)]);

    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    assert!(TypeChecker::resolver_struct_field_metadata(
        symbols
            .lookup(Namespace::Type, "Point")
            .expect("Point symbol")
    )
    .is_none());
}

#[test]
fn resolver_enum_variant_name_metadata_requires_variant_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let variants = TypeChecker::resolver_enum_variant_name_metadata(
        symbols
            .lookup(Namespace::Type, "Option")
            .expect("Option symbol"),
    )
    .expect("complete resolver variants");

    assert_eq!(variants, ["Some", "None"]);

    symbols.set_variant_names_for_test(Namespace::Type, "Option", None);
    assert!(TypeChecker::resolver_enum_variant_name_metadata(
        symbols
            .lookup(Namespace::Type, "Option")
            .expect("Option symbol")
    )
    .is_none());
}

#[test]
fn resolver_behavior_method_metadata_requires_method_types() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let methods = TypeChecker::resolver_behavior_method_metadata(
        symbols
            .lookup(Namespace::Behavior, "Json")
            .expect("Json symbol"),
    )
    .expect("complete resolver behavior methods");

    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].name, "encode");
    assert_eq!(methods[0].parameter_types, [AstType::SelfType]);
    assert_eq!(methods[0].return_type, AstType::Str);

    symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Json", None);
    assert!(TypeChecker::resolver_behavior_method_metadata(
        symbols
            .lookup(Namespace::Behavior, "Json")
            .expect("Json symbol")
    )
    .is_none());
}

#[test]
fn resolver_type_parameter_metadata_requires_names_and_bound_refs() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

identity<T: Json<T>> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let metadata = resolver_type_parameter_metadata(
        symbols
            .lookup(Namespace::Value, "identity")
            .expect("identity symbol"),
    )
    .expect("complete resolver type-parameter metadata");

    assert_eq!(metadata.names, ["T"]);
    assert_eq!(metadata.bound_refs.len(), 1);
    assert_eq!(metadata.bound_refs[0].type_parameter, "T");
    assert_eq!(metadata.bound_refs[0].behavior, "Json");
    assert_eq!(
        metadata.bound_refs[0].type_args,
        [AstType::Named("T".to_string())]
    );

    symbols.set_type_parameter_bound_refs_for_test(Namespace::Value, "identity", None);
    assert!(resolver_type_parameter_metadata(
        symbols
            .lookup(Namespace::Value, "identity")
            .expect("identity symbol")
    )
    .is_none());
}
