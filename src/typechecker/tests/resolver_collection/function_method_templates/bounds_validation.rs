use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_function_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
identity<T: Json<T>> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Function { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored function bounds should avoid stale AST generic-bound diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_type_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box<T: Json<T>>: { value: T }
Option<T: Json<T>>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("MissingBox".to_string());
        type_params[0].constraint_type_args.clear();
    }
    if let Declaration::Enum { type_params, .. } = &mut program.declarations[2] {
        type_params[0].constraint = Some("MissingOption".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored type bounds should avoid stale AST generic-bound diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_type_bounds() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box<T: Json<T>>: { value: T }
Option<T: Json<T>>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Type, "Box", None);
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Type, "Option", None);
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("MissingBox".to_string());
        type_params[0].constraint_type_args.clear();
    }
    if let Declaration::Enum { type_params, .. } = &mut program.declarations[2] {
        type_params[0].constraint = Some("MissingOption".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.structs
                .get("Box")
                .expect("struct info")
                .type_param_bounds
                .is_empty(),
            "resolver-backed struct collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
    assert!(
            tc.enums
                .get("Option")
                .expect("enum info")
                .type_param_bounds
                .is_empty(),
            "resolver-backed enum collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior bounds should avoid stale AST generic-bound diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_bounds() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bound_refs_for_test(Namespace::Behavior, "Serializable", None);
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[1] {
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Serializable").expect("behavior info");
    assert!(
            info.type_param_bounds.is_empty(),
            "resolver-backed behavior collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_bounds_for_validation() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Box: { value: i32 }
Box.impl = {
    keep<T: Json<T>> = (self: Box, value: T) T { value }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { type_params, .. } = &mut methods[0] {
            type_params[0].constraint = Some("Missing".to_string());
            type_params[0].constraint_type_args.clear();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method bounds should avoid stale AST generic-bound diagnostics: {:?}",
            tc.diagnostics
        );
}
