use super::*;

mod default_methods;
mod restored_signatures;

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_name_metadata() {
    let mut program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { name, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.behaviors.contains_key("Json"));
    assert!(!tc.behaviors.contains_key("Missing"));
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_methods() {
    let mut program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[0] {
        methods[0].params[1].ty = AstType::Named("Stale".to_string());
        methods[0].return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should not keep AST-only behavior methods when resolver method metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_behavior_default_body_refs_when_methods_incomplete(
) {
    let mut program = parse_program(
        r#"
Mapper<T>: behavior {
    map: (Self, value: T) T {
        same: T = value
        same
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should remove behavior methods when resolver method metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST behavior default body refs when resolver method metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_behavior_methods_after_name_restore() {
    let mut program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
    if let Declaration::Behavior { name, methods, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
        methods[0].params[1].ty = AstType::Named("Stale".to_string());
        methods[0].return_type = Some(AstType::Named("AlsoStale".to_string()));
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behaviors.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST behavior key after resolver name restoration"
        );
    assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should clear the restored behavior key when resolver method metadata is incomplete"
        );
}
