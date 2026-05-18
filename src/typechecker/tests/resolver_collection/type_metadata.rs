use super::*;

mod enum_metadata;

#[test]
fn collect_declarations_with_symbols_uses_resolver_struct_field_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) StaticString
}
Pipeline<T: Json<T>>: { callback: (i32) i32 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct {
        type_params,
        fields,
        ..
    } = &mut program.declarations[2]
    {
        type_params[0].constraint = Some("Debug".to_string());
        type_params[0].constraint_type_args.clear();
        fields[0].ty = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.structs.get("Pipeline").expect("struct info");
    assert_eq!(
        info.type_param_bounds.get("T"),
        Some(&BehaviorBound {
            behavior: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        })
    );
    assert_eq!(
        info.fields[0].1,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_struct_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Struct { name, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.structs.contains_key("Point"));
    assert!(!tc.structs.contains_key("Missing"));
}

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

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_struct_fields() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    if let Declaration::Struct { fields, .. } = &mut program.declarations[0] {
        fields[0].ty = AstType::Named("Stale".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.structs.contains_key("Point"),
            "resolver-backed collection should not keep AST-only struct fields when resolver field metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_struct_field_default_refs_when_fields_incomplete(
) {
    let mut program = parse_program(
        r#"
Box<T>: {
    value: T = {
        same: T = 1
        same
    }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(Namespace::Type, "Box", None);
    if let Declaration::Struct { type_params, .. } = &mut program.declarations[0] {
        type_params[0].name = "Stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.structs.contains_key("Box"),
            "resolver-backed collection should remove struct fields when resolver field metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST struct field default refs when resolver field metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_struct_fields_after_name_restore() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    if let Declaration::Struct { name, fields, .. } = &mut program.declarations[0] {
        *name = "Missing".to_string();
        fields[0].ty = AstType::Named("Stale".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.structs.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST struct key after resolver name restoration"
        );
    assert!(
            !tc.structs.contains_key("Point"),
            "resolver-backed collection should clear the restored struct key when resolver field metadata is incomplete"
        );
}
