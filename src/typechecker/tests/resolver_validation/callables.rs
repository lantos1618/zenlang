use super::*;

#[test]
fn callable_signature_insert_routes_function_and_method_keys() {
    let mut tc = TypeChecker::new();
    let function = FuncInfo {
        name: "make".to_string(),
        params: vec![],
        return_type: AstType::I32,
        type_params: vec![],
        type_param_bounds: HashMap::new(),
    };
    let method = FuncInfo {
        name: "Point.get".to_string(),
        params: vec![("self".to_string(), AstType::Named("Point".to_string()))],
        return_type: AstType::I32,
        type_params: vec![],
        type_param_bounds: HashMap::new(),
    };

    tc.insert_callable_signature("make", function);
    tc.insert_callable_signature("Point.get", method);

    assert!(tc.functions.contains_key("make"));
    assert!(!tc.methods.contains_key("make"));
    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.functions.contains_key("Point.get"));
}

#[test]
fn generic_callable_template_mut_routes_function_and_method_keys() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

identity<T> = (value: T) T { value }

Box.get<T> = (self: Box<T>) T { self.value }
"#,
    );
    let ast::Declaration::Function {
        type_params: function_type_params,
        params: function_params,
        return_type: function_return_type,
        body: function_body,
        span: function_span,
        ..
    } = &program.declarations[1]
    else {
        panic!("expected generic function");
    };
    let ast::Declaration::Method {
        type_params: method_type_params,
        params: method_params,
        return_type: method_return_type,
        body: method_body,
        span: method_span,
        ..
    } = &program.declarations[2]
    else {
        panic!("expected generic method");
    };
    let mut tc = TypeChecker::new();
    tc.generic_functions.insert(
        "identity".to_string(),
        generic_template_from_type_params(
            function_type_params,
            function_params,
            function_return_type,
            function_body,
            *function_span,
        )
        .expect("generic function template"),
    );
    tc.generic_methods.insert(
        "Box.get".to_string(),
        generic_template_from_type_params(
            method_type_params,
            method_params,
            method_return_type,
            method_body,
            *method_span,
        )
        .expect("generic method template"),
    );

    tc.generic_callable_template_mut("identity")
        .expect("function template")
        .return_type = Some(AstType::I32);
    tc.generic_callable_template_mut("Box.get")
        .expect("method template")
        .return_type = Some(AstType::Bool);

    assert_eq!(
        tc.generic_functions
            .get("identity")
            .and_then(|template| template.return_type.as_ref()),
        Some(&AstType::I32)
    );
    assert_eq!(
        tc.generic_methods
            .get("Box.get")
            .and_then(|template| template.return_type.as_ref()),
        Some(&AstType::Bool)
    );
    assert!(!tc.generic_methods.contains_key("identity"));
    assert!(!tc.generic_functions.contains_key("Box.get"));
}

#[test]
fn callable_template_rekey_routes_function_and_method_keys() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

identity<T> = (value: T) T { value }

Box.get<T> = (self: Box<T>) T { self.value }
"#,
    );
    let ast::Declaration::Function {
        type_params: function_type_params,
        params: function_params,
        return_type: function_return_type,
        body: function_body,
        span: function_span,
        ..
    } = &program.declarations[1]
    else {
        panic!("expected generic function");
    };
    let ast::Declaration::Method {
        type_params: method_type_params,
        params: method_params,
        return_type: method_return_type,
        body: method_body,
        span: method_span,
        ..
    } = &program.declarations[2]
    else {
        panic!("expected generic method");
    };
    let mut tc = TypeChecker::new();
    tc.generic_functions.insert(
        "identity".to_string(),
        generic_template_from_type_params(
            function_type_params,
            function_params,
            function_return_type,
            function_body,
            *function_span,
        )
        .expect("generic function template"),
    );
    tc.generic_methods.insert(
        "Box.get".to_string(),
        generic_template_from_type_params(
            method_type_params,
            method_params,
            method_return_type,
            method_body,
            *method_span,
        )
        .expect("generic method template"),
    );

    tc.rekey_callable_template("identity", "renamed");
    tc.rekey_callable_template("Box.get", "Box.fetch");

    assert!(tc.generic_functions.contains_key("renamed"));
    assert!(!tc.generic_functions.contains_key("identity"));
    assert!(tc.generic_methods.contains_key("Box.fetch"));
    assert!(!tc.generic_methods.contains_key("Box.get"));
}

#[test]
fn resolver_backed_callable_template_collection_defers_signature_metadata_to_resolver() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

identity<T> = (mut value: T) T { value }

Box.get<T> = (self: Box<T>, mut fallback: T) T { fallback }
"#,
    );
    let mut tc = TypeChecker::new();

    tc.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&program.declarations);
    });

    let function_template = tc
        .generic_functions
        .get("identity")
        .expect("function template stub");
    assert!(
            function_template.type_params.is_empty(),
            "resolver-backed generic function templates should not keep AST generic names before resolver metadata"
        );
    assert_eq!(function_template.params.len(), 1);
    assert_eq!(function_template.params[0].name, "");
    assert_eq!(function_template.params[0].ty, AstType::Void);
    assert!(function_template.params[0].mutable);
    assert_eq!(function_template.return_type, None);

    let method_template = tc
        .generic_methods
        .get("Box.get")
        .expect("method template stub");
    assert!(
            method_template.type_params.is_empty(),
            "resolver-backed generic method templates should not keep AST generic names before resolver metadata"
        );
    assert_eq!(method_template.params.len(), 2);
    assert_eq!(method_template.params[1].name, "");
    assert_eq!(method_template.params[1].ty, AstType::Void);
    assert!(method_template.params[1].mutable);
    assert_eq!(method_template.return_type, None);
}
