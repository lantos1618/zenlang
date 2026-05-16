use super::*;

#[test]
fn method_key_formats_type_qualified_method_name() {
    assert_eq!(TypeChecker::method_key("Point", "encode"), "Point.encode");
}

#[test]
fn resolver_behavior_ref_owner_prefers_exact_then_unique_fallbacks() {
    let tc = TypeChecker::new();
    let mut refs_by_type = HashMap::from([
        (
            "Point".to_string(),
            VecDeque::from(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![AstType::I32],
            }]),
        ),
        (
            "Label".to_string(),
            VecDeque::from(vec![BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            }]),
        ),
    ]);
    let missing_refs = HashSet::new();

    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Json", &[AstType::I32]),
        Some("Point".to_string())
    );
    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
        None
    );

    refs_by_type.remove("Label");
    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
        Some("Point".to_string())
    );

    refs_by_type.clear();
    let missing_refs = HashSet::from(["Recovered".to_string()]);
    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
        Some("Recovered".to_string())
    );
}

#[test]
fn resolver_symbol_metadata_helper_requires_symbol_and_selected_metadata() {
    let program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    assert_eq!(
        TypeChecker::resolver_symbol_metadata(&symbols, Namespace::Type, "Point", |symbol| {
            symbol.field_types.as_ref()
        })
        .map(|(_, fields)| fields[0].0.as_str()),
        Some("x")
    );
    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    assert!(
        TypeChecker::resolver_symbol_metadata(&symbols, Namespace::Type, "Point", |symbol| symbol
            .field_types
            .as_ref())
        .is_none()
    );
    assert!(TypeChecker::resolver_symbol_metadata(
        &symbols,
        Namespace::Type,
        "Missing",
        |symbol| symbol.field_types.as_ref()
    )
    .is_none());
}

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
    encode: (Self) str
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
