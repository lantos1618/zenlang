use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_struct_field_names_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { fields, .. } = &mut program.declarations[0] {
        fields[0].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed default validation should use resolver-restored field names: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_struct_field_defaults_validate_from_semantic_tasks() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let tasks =
        TypeChecker::collect_resolver_declaration_semantic_validation_tasks(&program.declarations);
    let mut stale_declarations = program.declarations.clone();
    if let Declaration::Struct { fields, .. } = &mut stale_declarations[0] {
        fields.clear();
    }
    let mut tc = TypeChecker::new();
    tc.with_resolver_backed_collection(|checker| checker.collect_declarations(&stale_declarations));
    let metadata_tasks =
        TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    tc.collect_resolver_declaration_metadata(&symbols, &metadata_tasks);

    tc.with_resolver_backed_collection(|checker| {
        checker.validate_resolver_declaration_semantics_from_semantic_tasks(&tasks, Some(&symbols));
    });

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed default validation should use semantic validation tasks: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_backed_struct_field_defaults_use_semantic_tasks() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let mut stale_declarations = program.declarations.clone();
    if let Declaration::Struct { fields, .. } = &mut stale_declarations[0] {
        fields.clear();
    }
    let mut tc = TypeChecker::new();
    tc.with_resolver_backed_collection(|checker| checker.collect_declarations(&stale_declarations));
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    tc.collect_resolver_declaration_metadata(&symbols, &tasks);

    tc.with_resolver_backed_collection(|checker| {
        checker.validate_struct_field_defaults(&program.declarations, Some(&symbols));
    });

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed default validation should use focused semantic tasks: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_backed_semantic_validation_uses_semantic_tasks() {
    let program = parse_program(
        r#"
Point: { x: i32 = true }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    let mut tc = TypeChecker::new();
    tc.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&program.declarations)
    });
    tc.collect_resolver_declaration_metadata(&symbols, &tasks);

    tc.with_resolver_backed_collection(|checker| {
        checker.validate_collected_declaration_semantics(&program.declarations, Some(&symbols));
    });

    assert!(
        tc.diagnostics.iter().any(|diag| {
            diag.code == "E3073"
                && diag
                    .message
                    .contains("field `x` default expects `i32`, found `bool`")
        }),
        "resolver-backed semantic validation should use resolver semantic tasks: {:?}",
        tc.diagnostics
    );
}
