use super::*;

#[test]
fn resolver_behavior_ref_queue_selection_prefers_exact_then_front() {
    let refs = VecDeque::from(vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        },
        BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        },
    ]);

    assert_eq!(
        TypeChecker::resolver_behavior_ref_queue_index(&refs, "Debug"),
        Some(1)
    );
    assert_eq!(
        TypeChecker::resolver_behavior_ref_queue_index(&refs, "Missing"),
        Some(0)
    );
    assert_eq!(
        TypeChecker::resolver_behavior_ref_queue_index(&VecDeque::new(), "Missing"),
        None
    );
}

#[test]
fn named_queue_selection_prefers_exact_then_front() {
    let items = VecDeque::from(["Json".to_string(), "Debug".to_string()]);

    assert_eq!(
        TypeChecker::named_queue_index(&items, "Debug", String::as_str),
        Some(1)
    );
    assert_eq!(
        TypeChecker::named_queue_index(&items, "Missing", String::as_str),
        Some(0)
    );
    assert_eq!(
        TypeChecker::named_queue_index(&VecDeque::<String>::new(), "Missing", String::as_str),
        None
    );
}

#[test]
fn named_queue_selection_can_preserve_front_for_future_match() {
    let items = VecDeque::from(["Json".to_string(), "Debug".to_string()]);

    assert_eq!(
        TypeChecker::named_queue_index_preserving_future_front(
            &items,
            "Debug",
            Vec::<&str>::new(),
            String::as_str,
        ),
        Some(1)
    );
    assert_eq!(
        TypeChecker::named_queue_index_preserving_future_front(
            &items,
            "Missing",
            ["Json"],
            String::as_str,
        ),
        None
    );
    assert_eq!(
        TypeChecker::named_queue_index_preserving_future_front(
            &items,
            "Missing",
            ["Other"],
            String::as_str,
        ),
        Some(0)
    );
}

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
fn resolver_backed_impl_method_key_requires_resolver_collection() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let (span, ast_key) = if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function { name, span, .. } = &mut methods[0] {
            *name = "missing".to_string();
            (*span, TypeChecker::method_key(type_name, name))
        } else {
            panic!("expected impl method");
        }
    } else {
        panic!("expected impl block");
    };
    let mut tc = TypeChecker::new();

    assert_eq!(
        tc.resolver_backed_impl_method_key(Some(&symbols), &ast_key, "Missing", span),
        None
    );
    tc.resolver_backed_collection = true;
    assert_eq!(
        tc.resolver_backed_impl_method_key(Some(&symbols), &ast_key, "Missing", span),
        Some("Point.encode".to_string())
    );
}

#[test]
fn effective_behavior_impl_methods_carry_named_declaration_and_method_name() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
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

#[test]
fn behavior_default_synthesis_skip_requires_resolver_collection_and_missing_impl_ref() {
    let mut tc = TypeChecker::new();
    tc.resolver_missing_behavior_impl_refs
        .insert("Point".to_string());

    assert!(!tc.should_skip_behavior_default_synthesis("Point"));
    tc.resolver_backed_collection = true;
    assert!(tc.should_skip_behavior_default_synthesis("Point"));
    assert!(!tc.should_skip_behavior_default_synthesis("Other"));
}

#[test]
fn resolver_backed_behavior_collection_defers_generic_metadata_to_resolver() {
    let program = parse_program(
        r#"
Json<T: Json<T>>: behavior {
    encode: (Self) T {
        1
    }
}
"#,
    );
    let mut tc = TypeChecker::new();

    tc.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&program.declarations);
    });

    let behavior = tc.behaviors.get("Json").expect("behavior stub");
    assert!(
            behavior.type_params.is_empty(),
            "resolver-backed behavior collection should not keep AST generic names before resolver metadata"
        );
    assert!(
            behavior.type_param_bounds.is_empty(),
            "resolver-backed behavior collection should not keep AST generic bounds before resolver metadata"
        );
    assert!(
            behavior.methods[0].default_body.is_some(),
            "resolver-backed behavior collection should still keep default bodies for later resolver metadata restoration"
        );
}
