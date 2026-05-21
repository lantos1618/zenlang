use super::*;

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
