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
    assert_eq!(parameter.display, "(i32) StaticString");
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
