use super::*;

#[test]
fn resolver_params_from_metadata_preserves_ast_param_shape() {
    let existing_span = Span::new(1, 10, 15);
    let default_span = Span::new(1, 20, 25);
    let existing_params = vec![Param {
        name: "stale".to_string(),
        ty: AstType::Bool,
        mutable: true,
        span: existing_span,
    }];
    let parameter_names = vec!["value".to_string(), "mapper".to_string()];
    let parameter_types = vec![
        AstType::I32,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        },
    ];

    let params = TypeChecker::resolver_params_from_metadata(
        &existing_params,
        &parameter_names,
        &parameter_types,
        default_span,
    );

    assert_eq!(params[0].name, "value");
    assert_eq!(params[0].ty, AstType::I32);
    assert!(params[0].mutable);
    assert_eq!(params[0].span, existing_span);
    assert_eq!(params[1].name, "mapper");
    assert_eq!(params[1].ty, parameter_types[1]);
    assert!(!params[1].mutable);
    assert_eq!(params[1].span, default_span);
}

#[test]
fn resolver_optional_return_type_maps_void_to_missing_annotation() {
    assert_eq!(
        TypeChecker::resolver_optional_return_type(&AstType::Void),
        None
    );
    assert_eq!(
        TypeChecker::resolver_optional_return_type(&AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        }),
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        })
    );
}

#[test]
fn resolver_enum_variants_from_metadata_uses_owner_scoped_payloads() {
    let program = parse_program(
        r#"
First: Wrap(i32), None
Second: Wrap(StaticString)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let variants = vec!["Wrap".to_string(), "None".to_string()];

    assert_eq!(
        TypeChecker::resolver_enum_variants_from_metadata(&symbols, "First", &variants),
        vec![
            ("Wrap".to_string(), Some(AstType::I32)),
            ("None".to_string(), None),
        ]
    );
    assert_eq!(
        TypeChecker::resolver_enum_variants_from_metadata(
            &symbols,
            "Second",
            &["Wrap".to_string()]
        ),
        vec![("Wrap".to_string(), Some(AstType::Str))]
    );
}

#[test]
fn resolver_struct_fields_from_metadata_restores_field_names_and_defaults() {
    let fields = vec![
        ("x".to_string(), AstType::I32),
        (
            "callback".to_string(),
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::Str),
            },
        ),
    ];
    let ast_fields = vec![
        StructField {
            name: "stale_x".to_string(),
            ty: AstType::Bool,
            default: Some(Expression::BoolLiteral {
                value: true,
                span: Span::dummy(),
            }),
            mutable: false,
            span: Span::dummy(),
        },
        StructField {
            name: "stale_callback".to_string(),
            ty: AstType::Bool,
            default: None,
            mutable: false,
            span: Span::dummy(),
        },
    ];

    let (restored_fields, defaults) =
        TypeChecker::resolver_struct_fields_from_metadata(&fields, &ast_fields);

    assert_eq!(restored_fields, fields);
    assert!(defaults.contains_key("x"));
    assert!(!defaults.contains_key("stale_x"));
    assert!(!defaults.contains_key("callback"));
}

#[test]
fn resolver_behavior_methods_from_metadata_preserves_defaults_by_resolver_order() {
    let first_span = Span::new(1, 10, 15);
    let second_span = Span::new(1, 20, 25);
    let default_span = Span::new(1, 30, 35);
    let existing_methods = vec![
        ast::BehaviorMethod {
            name: "first".to_string(),
            params: vec![Param {
                name: "stale".to_string(),
                ty: AstType::Bool,
                mutable: false,
                span: first_span,
            }],
            return_type: Some(AstType::Bool),
            default_body: Some(Expression::IntLiteral {
                value: 1,
                span: first_span,
            }),
            span: first_span,
        },
        ast::BehaviorMethod {
            name: "second".to_string(),
            params: vec![],
            return_type: None,
            default_body: Some(Expression::IntLiteral {
                value: 2,
                span: second_span,
            }),
            span: second_span,
        },
    ];
    let method_types = vec![
        BehaviorMethodTypeMetadata {
            name: "second".to_string(),
            parameter_names: vec!["value".to_string()],
            parameter_types: vec![AstType::I32],
            return_type: AstType::Str,
        },
        BehaviorMethodTypeMetadata {
            name: "first".to_string(),
            parameter_names: vec![],
            parameter_types: vec![],
            return_type: AstType::Void,
        },
    ];

    let methods = TypeChecker::resolver_behavior_methods_from_metadata(
        existing_methods,
        &method_types,
        default_span,
    );

    assert_eq!(methods[0].name, "second");
    assert_eq!(methods[0].params[0].name, "value");
    assert_eq!(methods[0].params[0].ty, AstType::I32);
    assert_eq!(methods[0].return_type, Some(AstType::Str));
    assert_eq!(methods[0].span, second_span);
    assert!(matches!(
        methods[0].default_body,
        Some(Expression::IntLiteral { value: 2, .. })
    ));
    assert_eq!(methods[1].name, "first");
    assert_eq!(methods[1].return_type, None);
    assert_eq!(methods[1].span, first_span);
    assert!(matches!(
        methods[1].default_body,
        Some(Expression::IntLiteral { value: 1, .. })
    ));
}

#[test]
fn behavior_parent_refs_from_metadata_restores_keys_and_type_args() {
    let tc = TypeChecker::new();
    let metadata = vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        },
        BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        },
    ];

    let refs = tc.behavior_parent_refs_from_metadata(&metadata);

    assert_eq!(refs[0].behavior, "Json");
    assert_eq!(refs[0].type_args, vec![AstType::Named("T".to_string())]);
    assert_eq!(refs[0].key, "Json_T");
    assert_eq!(refs[1].behavior, "Debug");
    assert!(refs[1].type_args.is_empty());
    assert_eq!(refs[1].key, "Debug");
}

#[test]
fn behavior_impl_refs_from_metadata_restores_type_and_behavior_keys() {
    let tc = TypeChecker::new();
    let metadata = vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::Str],
        },
        BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        },
    ];

    assert_eq!(
        tc.behavior_impl_refs_from_metadata("Point", &metadata),
        vec![
            ("Point".to_string(), "Json_StaticString".to_string()),
            ("Point".to_string(), "Debug".to_string()),
        ]
    );
}
