use super::*;

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
