use super::*;

#[test]
fn expected_behavior_symbol_builds_type_like_and_methods_together() {
    let type_params = vec![ast::TypeParam {
        name: "T".to_string(),
        constraint: None,
        constraint_type_args: vec![],
        span: Span::dummy(),
    }];
    let methods = vec![ast::BehaviorMethod {
        name: "encode".to_string(),
        params: vec![Param {
            name: "value".to_string(),
            ty: AstType::Named("Self".to_string()),
            mutable: false,
            span: Span::dummy(),
        }],
        return_type: Some(AstType::Named("T".to_string())),
        default_body: None,
        span: Span::dummy(),
    }];

    let symbol = ExpectedBehaviorSymbol::new(&type_params, &methods, true);

    assert_eq!(symbol.type_like.is_public, Some(true));
    assert_eq!(symbol.type_like.type_params[0].name, "T");
    assert_eq!(symbol.methods[0].signature.0, "encode");
    assert_eq!(symbol.methods[0].signature.1, vec!["Self".to_string()]);
    assert_eq!(symbol.methods[0].signature.2, "T");
    assert_eq!(symbol.methods[0].metadata.name, "encode");
    assert_eq!(
        symbol.methods[0].metadata.parameter_names,
        vec!["value".to_string()]
    );
    assert_eq!(
        symbol.methods[0].metadata.return_type,
        AstType::Named("T".to_string())
    );
}

#[test]
fn expected_struct_symbol_builds_type_like_and_fields_together() {
    let type_params = vec![ast::TypeParam {
        name: "T".to_string(),
        constraint: None,
        constraint_type_args: vec![],
        span: Span::dummy(),
    }];
    let fields = vec![StructField {
        name: "value".to_string(),
        ty: AstType::Named("T".to_string()),
        default: None,
        mutable: false,
        span: Span::dummy(),
    }];

    let symbol = ExpectedStructSymbol::new(&type_params, &fields, true);

    assert_eq!(symbol.type_like.is_public, Some(true));
    assert_eq!(symbol.type_like.type_params[0].name, "T");
    assert_eq!(
        symbol.fields[0].display,
        ("value".to_string(), "T".to_string())
    );
    assert_eq!(
        symbol.fields[0].typed,
        ("value".to_string(), AstType::Named("T".to_string()))
    );
}

#[test]
fn expected_enum_symbol_builds_type_like_and_variants_together() {
    let type_params = vec![ast::TypeParam {
        name: "T".to_string(),
        constraint: None,
        constraint_type_args: vec![],
        span: Span::dummy(),
    }];
    let variants = vec![
        EnumVariant {
            name: "Some".to_string(),
            payload: Some(AstType::Named("T".to_string())),
            span: Span::dummy(),
        },
        EnumVariant {
            name: "None".to_string(),
            payload: None,
            span: Span::dummy(),
        },
    ];

    let symbol = ExpectedEnumSymbol::new(&type_params, &variants, true);

    assert_eq!(symbol.type_like.is_public, Some(true));
    assert_eq!(symbol.type_like.type_params[0].name, "T");
    assert_eq!(
        symbol.variant_names,
        vec!["Some".to_string(), "None".to_string()]
    );
}

#[test]
fn expected_variant_symbol_builds_owner_visibility_and_payload_together() {
    let payload = Some(AstType::Function {
        params: vec![AstType::I32],
        ret: Box::new(AstType::Bool),
    });

    let symbol = ExpectedVariantSymbol::new("Result", true, &payload);

    assert_eq!(symbol.owner_name, "Result");
    assert!(symbol.is_public);
    assert_eq!(symbol.payload.display, Some("(i32) bool".to_string()));
    assert_eq!(
        symbol.payload.typed,
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Bool),
        })
    );
}
