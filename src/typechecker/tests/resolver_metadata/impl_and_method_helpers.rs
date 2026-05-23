use super::*;

mod resolver_backed_collection;

#[test]
fn impl_effective_method_name_prefers_resolver_then_ast_then_collected_signature() {
    let mut tc = TypeChecker::new();
    tc.resolver_backed_collection = true;
    tc.methods.insert(
        "Point.describe".to_string(),
        FuncInfo {
            name: "Point.describe".to_string(),
            params: Vec::new(),
            return_type: AstType::Void,
            type_params: Vec::new(),
            type_param_bounds: HashMap::new(),
        },
    );
    let mut unmatched = VecDeque::from([
        "encode".to_string(),
        "debug".to_string(),
        "describe".to_string(),
    ]);

    assert_eq!(
        tc.impl_effective_method_name(
            &mut unmatched,
            "stale",
            Some("Point.encode".to_string()),
            "Point",
        ),
        "encode"
    );
    assert_eq!(
        tc.impl_effective_method_name(&mut unmatched, "debug", None, "Point"),
        "debug"
    );
    assert_eq!(
        tc.impl_effective_method_name(&mut unmatched, "missing", None, "Point"),
        "describe"
    );
    assert!(unmatched.is_empty());
}

#[test]
fn effective_behavior_impl_methods_carry_named_declaration_and_method_name() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let Declaration::ImplBlock { methods, .. } = &program.declarations[2] else {
        panic!("expected impl block");
    };
    let mut tc = TypeChecker::new();
    tc.resolver_backed_collection = true;
    let mut unmatched = VecDeque::from(["encode".to_string()]);

    let effective =
        tc.effective_behavior_impl_methods(Some(&symbols), "Point", methods, &mut unmatched);

    assert_eq!(effective.len(), 1);
    assert_eq!(effective[0].method_name, "encode");
    assert!(matches!(
        effective[0].declaration,
        Declaration::Function { name, .. } if name == "encode"
    ));
    assert!(unmatched.is_empty());
}

#[test]
fn template_dependency_entries_use_named_fields() {
    let entry = TemplateDependencyEntry::<StructInfo> {
        name: "Point".to_string(),
        previous: None,
    };

    assert_eq!(entry.name, "Point");
    assert!(entry.previous.is_none());
}

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
