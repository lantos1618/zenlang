use super::*;

#[test]
fn resolver_backed_behavior_impl_method_signature_name_prefers_resolver_key() {
    let tc = TypeChecker::new();
    let mut required = VecDeque::from([
        ast::BehaviorMethod {
            name: "encode".to_string(),
            params: Vec::new(),
            return_type: Some(AstType::Str),
            default_body: None,
            span: Span::dummy(),
        },
        ast::BehaviorMethod {
            name: "debug".to_string(),
            params: Vec::new(),
            return_type: Some(AstType::Str),
            default_body: None,
            span: Span::dummy(),
        },
    ]);

    assert_eq!(
        tc.resolver_backed_behavior_impl_method_signature_name(
            &mut required,
            "stale",
            Some("Point.encode"),
            "Point",
        ),
        Some("encode".to_string())
    );
    assert_eq!(
        tc.resolver_backed_behavior_impl_method_signature_name(
            &mut required,
            "debug",
            None,
            "Point",
        ),
        Some("debug".to_string())
    );
    assert!(required.is_empty());
}

#[test]
fn resolver_backed_method_signature_requires_resolver_collection() {
    let mut tc = TypeChecker::new();
    tc.methods.insert(
        "Point.encode".to_string(),
        FuncInfo {
            name: "Point.encode".to_string(),
            params: Vec::new(),
            return_type: AstType::Str,
            type_params: Vec::new(),
            type_param_bounds: HashMap::new(),
        },
    );

    assert!(tc
        .resolver_backed_method_signature("Point", "encode")
        .is_none());
    tc.resolver_backed_collection = true;
    assert_eq!(
        tc.resolver_backed_method_signature("Point", "encode")
            .map(|info| info.return_type.clone()),
        Some(AstType::Str)
    );
}
