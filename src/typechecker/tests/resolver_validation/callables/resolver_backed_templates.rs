use super::*;

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
