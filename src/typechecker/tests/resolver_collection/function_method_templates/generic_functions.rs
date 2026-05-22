use super::*;

mod param_mutability;

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) StaticString
}
apply<T: Json<T>> = (callback: (T) T) (T) T {
    callback
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        type_params,
        params,
        return_type,
        ..
    } = &mut program.declarations[2]
    {
        type_params[0].name = "Stale".to_string();
        type_params[0].constraint = Some("Debug".to_string());
        type_params[0].constraint_type_args.clear();
        params[0].ty = AstType::I32;
        *return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc.generic_functions.get("apply").expect("generic template");
    assert_eq!(template.type_params, vec!["T".to_string()]);
    assert_eq!(
        tc.functions
            .get("apply")
            .expect("function info")
            .type_param_bounds
            .get("T"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        })
    );
    assert_eq!(
        template.params[0].ty,
        AstType::Function {
            params: vec![AstType::Named("T".to_string())],
            ret: Box::new(AstType::Named("T".to_string())),
        }
    );
    assert_eq!(
        template.return_type,
        Some(AstType::Function {
            params: vec![AstType::Named("T".to_string())],
            ret: Box::new(AstType::Named("T".to_string())),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_name_for_body_type_refs() {
    let mut program = parse_program(
        r#"
keep<T> = (value: T) T {
    same: T = value
    same
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function {
        name, type_params, ..
    } = &mut program.declarations[0]
    {
        *name = "missing".to_string();
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored generic function name and type parameters should avoid stale AST body type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_clears_generic_function_template_type_params_when_resolver_bounds_missing(
) {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
identity<T: Json<T>> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Value, "identity", None);
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("identity")
        .expect("generic function template");
    assert!(
            template.type_params.is_empty(),
            "resolver-backed generic templates should not keep type parameter names when typed bound metadata is incomplete"
        );
    assert!(
            tc.functions
                .get("identity")
                .expect("function info")
                .type_params
                .is_empty(),
            "function info and template type parameter handoff should agree when resolver metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_return_presence() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T {
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { return_type, .. } = &mut program.declarations[0] {
        *return_type = None;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("identity")
        .expect("generic template");
    assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_parameter_count() {
    let mut program = parse_program(
        r#"
choose<T> = (left: T, right: T) T {
    left
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { params, .. } = &mut program.declarations[0] {
        params.pop();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_functions
        .get("choose")
        .expect("generic template");
    assert_eq!(template.params.len(), 2);
    assert_eq!(template.params[0].name, "left");
    assert_eq!(template.params[1].name, "right");
    assert_eq!(template.params[0].ty, AstType::Named("T".to_string()));
    assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
}
