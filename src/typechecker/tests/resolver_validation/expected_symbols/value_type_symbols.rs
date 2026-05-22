use super::*;

#[test]
fn expected_behavior_method_builds_signature_and_metadata_together() {
    let method = ast::BehaviorMethod {
        name: "map".to_string(),
        params: vec![Param {
            name: "mapper".to_string(),
            ty: AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::Str),
            },
            mutable: false,
            span: Span::dummy(),
        }],
        return_type: Some(AstType::Str),
        default_body: None,
        span: Span::dummy(),
    };

    let expected = ExpectedBehaviorMethod::new(&method);

    assert_eq!(
        expected.signature,
        (
            "map".to_string(),
            vec!["(i32) StaticString".to_string()],
            "StaticString".to_string(),
        )
    );
    assert_eq!(expected.metadata.name, "map");
    assert_eq!(
        expected.metadata.parameter_names,
        vec!["mapper".to_string()]
    );
    assert_eq!(
        expected.metadata.parameter_types,
        vec![AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        }]
    );
    assert_eq!(expected.metadata.return_type, AstType::Str);
}

#[test]
fn expected_value_signature_builds_components_together() {
    let params = vec![Param {
        name: "value".to_string(),
        ty: AstType::Named("T".to_string()),
        mutable: false,
        span: Span::dummy(),
    }];
    let return_type = Some(AstType::Named("T".to_string()));
    let type_params = vec![ast::TypeParam {
        name: "T".to_string(),
        constraint: Some("Json".to_string()),
        constraint_type_args: vec![AstType::Named("T".to_string())],
        span: Span::dummy(),
    }];

    let signature = ExpectedValueSignature::new(&params, &return_type, &type_params);

    assert_eq!(signature.params[0].name, "value");
    assert_eq!(signature.params[0].display, "T");
    assert_eq!(signature.params[0].typed, AstType::Named("T".to_string()));
    assert_eq!(signature.return_type.display, "T");
    assert_eq!(signature.return_type.typed, AstType::Named("T".to_string()));
    assert_eq!(signature.type_params[0].name, "T");
    let bound = signature.type_params[0]
        .bound
        .as_ref()
        .expect("expected bound");
    assert_eq!(bound.display, ("T".to_string(), "Json<T>".to_string()));
    assert_eq!(bound.reference.behavior, "Json");
    assert_eq!(
        bound.reference.type_args,
        vec![AstType::Named("T".to_string())]
    );
}

#[test]
fn expected_value_symbol_builds_signature_and_visibility_together() {
    let params = vec![Param {
        name: "value".to_string(),
        ty: AstType::I32,
        mutable: false,
        span: Span::dummy(),
    }];
    let return_type = Some(AstType::Bool);

    let symbol = ExpectedValueSymbol::new(&params, &return_type, &[], true);

    assert!(symbol.is_public);
    assert_eq!(symbol.signature.params[0].name, "value");
    assert_eq!(symbol.signature.params[0].display, "i32");
    assert_eq!(symbol.signature.params[0].typed, AstType::I32);
    assert_eq!(symbol.signature.return_type.display, "bool");
    assert_eq!(symbol.signature.return_type.typed, AstType::Bool);
    assert!(symbol.signature.type_params.is_empty());
}

#[test]
fn expected_type_like_symbol_builds_type_params_and_visibility_together() {
    let type_params = vec![ast::TypeParam {
        name: "T".to_string(),
        constraint: Some("Json".to_string()),
        constraint_type_args: vec![AstType::Named("T".to_string())],
        span: Span::dummy(),
    }];

    let symbol = ExpectedTypeLikeSymbol::new(&type_params, Some(true));

    assert_eq!(symbol.is_public, Some(true));
    assert_eq!(symbol.type_params[0].name, "T");
    let bound = symbol.type_params[0]
        .bound
        .as_ref()
        .expect("expected bound");
    assert_eq!(bound.display, ("T".to_string(), "Json<T>".to_string()));
    assert_eq!(bound.reference.type_parameter, "T");
    assert_eq!(bound.reference.behavior, "Json");
    assert_eq!(
        bound.reference.type_args,
        vec![AstType::Named("T".to_string())]
    );
}
