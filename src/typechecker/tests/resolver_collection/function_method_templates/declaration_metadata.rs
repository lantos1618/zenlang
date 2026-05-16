use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_name_metadata() {
    let mut program = parse_program(
        r#"
main = () i32 { 1 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { name, .. } = &mut program.declarations[0] {
        *name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.functions.contains_key("main"));
    assert!(!tc.functions.contains_key("missing"));
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_generic_function_template_name_metadata() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { name, .. } = &mut program.declarations[0] {
        *name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.generic_functions.contains_key("identity"));
    assert!(!tc.generic_functions.contains_key("missing"));
}

#[test]
fn collect_declarations_with_symbols_clears_stale_generic_function_template_after_name_restore() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "identity", None);
    if let Declaration::Function {
        name,
        params,
        return_type,
        ..
    } = &mut program.declarations[0]
    {
        *name = "missing".to_string();
        params[0].ty = AstType::Named("Stale".to_string());
        *return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.generic_functions.contains_key("missing"),
            "resolver-backed collection should clear the stale AST generic function template key after resolver name restoration"
        );
    assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should clear the restored generic function template key when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_type_params_for_type_refs() {
    let mut program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored function type parameters should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_metadata_for_type_refs() {
    let mut program = parse_program(
        r#"
Box<T>: { value: T }
Option<T>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "StaleBox".to_string();
    }
    if let Declaration::Enum { type_params, .. } = &mut program.declarations[1] {
        type_params[0].name = "StaleOption".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored type metadata should avoid stale AST type-ref diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_type_params_for_type_refs() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior type parameters should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
}
