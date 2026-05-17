use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_enum_payload_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Callback<T: Json<T>>: Wrap((i32) i32), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Enum {
        type_params,
        variants,
        ..
    } = &mut program.declarations[2]
    {
        type_params[0].constraint = Some("Debug".to_string());
        type_params[0].constraint_type_args.clear();
        variants[0].payload = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.enums.get("Callback").expect("enum info");
    assert_eq!(
        info.type_param_bounds.get("T"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        })
    );
    assert_eq!(
        info.variants[0].1,
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        })
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_enum_name_metadata() {
    let mut program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Enum { name, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.enums.contains_key("Option"));
    assert!(!tc.enums.contains_key("Missing"));
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_enum_variants() {
    let mut program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Option", None);
    if let Declaration::Enum { variants, .. } = &mut program.declarations[0] {
        variants[0].payload = Some(AstType::Named("Stale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.enums.contains_key("Option"),
            "resolver-backed collection should not keep AST-only enum variants when resolver variant metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_enum_variants_after_name_restore() {
    let mut program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Option", None);
    if let Declaration::Enum { name, variants, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
        variants[0].payload = Some(AstType::Named("Stale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.enums.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST enum key after resolver name restoration"
        );
    assert!(
            !tc.enums.contains_key("Option"),
            "resolver-backed collection should clear the restored enum key when resolver variant metadata is incomplete"
        );
}
