use super::*;

#[test]
fn expected_parameter_builds_name_display_and_type_together() {
    let parameter = ExpectedParameter::new(
        "mapper",
        &AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        },
    );

    assert_eq!(parameter.name, "mapper");
    assert_eq!(parameter.display, "(i32) str");
    assert_eq!(
        parameter.typed,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Str),
        }
    );
}

#[test]
fn expected_return_metadata_defaults_and_displays_together() {
    let explicit = ExpectedReturnMetadata::new(&Some(AstType::Named("Point".to_string())));
    let implicit = ExpectedReturnMetadata::new(&None);

    assert_eq!(explicit.display, "Point");
    assert_eq!(explicit.typed, AstType::Named("Point".to_string()));
    assert_eq!(implicit.display, "void");
    assert_eq!(implicit.typed, AstType::Void);
}

#[test]
fn expected_type_parameter_builds_bound_display_and_ref_together() {
    let type_param = ast::TypeParam {
        name: "T".to_string(),
        constraint: Some("Json".to_string()),
        constraint_type_args: vec![AstType::Named("T".to_string())],
        span: Span::dummy(),
    };

    let expected = ExpectedTypeParameter::new(&type_param);
    let bound = expected.bound.expect("expected bound");

    assert_eq!(expected.name, "T");
    assert_eq!(bound.display, ("T".to_string(), "Json<T>".to_string()));
    assert_eq!(bound.reference.type_parameter, "T");
    assert_eq!(bound.reference.behavior, "Json");
    assert_eq!(
        bound.reference.type_args,
        vec![AstType::Named("T".to_string())]
    );
}

#[test]
fn expected_field_builds_display_and_type_together() {
    let field = ExpectedField::new(
        "mapper",
        &AstType::Function {
            params: vec![AstType::Named("Input".to_string())],
            ret: Box::new(AstType::Named("Output".to_string())),
        },
    );

    assert_eq!(
        field.display,
        ("mapper".to_string(), "(Input) Output".to_string())
    );
    assert_eq!(
        field.typed,
        (
            "mapper".to_string(),
            AstType::Function {
                params: vec![AstType::Named("Input".to_string())],
                ret: Box::new(AstType::Named("Output".to_string())),
            }
        )
    );
}

#[test]
fn expected_variant_payload_builds_display_and_type_together() {
    let payload = ExpectedVariantPayloadType::new(&Some(AstType::Function {
        params: vec![AstType::I32],
        ret: Box::new(AstType::Bool),
    }));
    let empty_payload = ExpectedVariantPayloadType::new(&None);

    assert_eq!(payload.display, Some("(i32) bool".to_string()));
    assert_eq!(
        payload.typed,
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Bool),
        })
    );
    assert_eq!(empty_payload.display, None);
    assert_eq!(empty_payload.typed, None);
}

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
            vec!["(i32) str".to_string()],
            "str".to_string(),
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

#[test]
fn expected_import_symbol_builds_source_and_visibility_together() {
    let symbol = ExpectedImportSymbol::new("std.io");

    assert_eq!(symbol.source, "std.io");
    assert!(!symbol.is_public);
}

#[test]
fn expected_module_symbol_builds_name_source_and_visibility_together() {
    let symbol = ExpectedModuleSymbol::new("std.io");

    assert_eq!(symbol.name, "std.io");
    assert_eq!(symbol.source, None);
    assert!(!symbol.is_public);
}

#[test]
fn expected_local_symbol_builds_scope_mutability_source_and_visibility_together() {
    let symbol = ExpectedLocalSymbol::new(true, 42);

    assert_eq!(symbol.scope_id, 42);
    assert!(symbol.is_mutable);
    assert_eq!(symbol.source, None);
    assert!(!symbol.is_public);
}

#[test]
fn expected_behavior_edge_builds_display_and_metadata_together() {
    let edge = ExpectedBehaviorEdge::new("Json", &[AstType::I32]);

    assert_eq!(edge.display, "Json<i32>");
    assert_eq!(edge.metadata.name, "Json");
    assert_eq!(edge.metadata.type_args, vec![AstType::I32]);
}

#[test]
fn expected_behavior_associations_build_impl_and_required_edges_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );

    let expected = ExpectedBehaviorAssociations::new(&program);
    let impl_edge = &expected.impls.edges_for("Point")[0];
    let required_edge = &expected.required.edges_for("Point")[0];

    assert_eq!(impl_edge.display, "Json<str>");
    assert_eq!(impl_edge.metadata.name, "Json");
    assert_eq!(impl_edge.metadata.type_args, vec![AstType::Str]);
    assert_eq!(required_edge.display, "Json<str>");
    assert_eq!(required_edge.metadata.name, "Json");
    assert_eq!(required_edge.metadata.type_args, vec![AstType::Str]);
}
