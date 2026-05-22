use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_name_metadata()
{
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { value }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params.pop();
            *return_type = None;
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic impl method template");
    assert!(!tc.generic_methods.contains_key("Box.missing"));
    assert_eq!(template.params.len(), 2);
    assert_eq!(template.params[0].name, "self");
    assert_eq!(template.params[1].name, "value");
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) StaticString
}
Box: { value: i32 }

Box.impl = {
    apply<U: Json<U>> = (self: Box, callback: (U) U) (U) U {
        callback
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[3] {
        if let Declaration::Function {
            type_params,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            type_params[0].name = "Stale".to_string();
            type_params[0].constraint = Some("Debug".to_string());
            type_params[0].constraint_type_args.clear();
            params[1].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.apply")
        .expect("generic impl method template");
    assert_eq!(template.type_params, vec!["U".to_string()]);
    assert_eq!(
        tc.methods
            .get("Box.apply")
            .expect("impl method info")
            .type_param_bounds
            .get("U"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("U".to_string())],
        })
    );
    assert_eq!(
        template.params[1].ty,
        AstType::Function {
            params: vec![AstType::Named("U".to_string())],
            ret: Box::new(AstType::Named("U".to_string())),
        }
    );
    assert_eq!(
        template.return_type,
        Some(AstType::Function {
            params: vec![AstType::Named("U".to_string())],
            ret: Box::new(AstType::Named("U".to_string())),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_return_presence(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T {
        value
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { return_type, .. } = &mut methods[0] {
            *return_type = None;
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.keep")
        .expect("generic impl method template");
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_parameter_count(
) {
    let mut program = parse_program(
        r#"
Box: { value: i32 }

Box.impl = {
    choose<T> = (self: Box, left: T, right: T) T {
        left
    }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params.pop();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Box.choose")
        .expect("generic impl method template");
    assert_eq!(template.params.len(), 3);
    assert_eq!(template.params[0].name, "self");
    assert_eq!(template.params[1].name, "left");
    assert_eq!(template.params[2].name, "right");
    assert_eq!(template.params[0].ty, AstType::Named("Box".to_string()));
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
    assert_eq!(template.params[2].ty, AstType::Named("T".to_string()));
}
