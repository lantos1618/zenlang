use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].name, "encode");
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method name metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_return_presence_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].return_type = None;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].return_type, Some(AstType::Str));
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method return metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_count() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) StaticString
    describe: (Self) StaticString
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
    describe = (value: Point) StaticString { "desc" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods.pop();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods.len(), 2);
    assert_eq!(info.methods[0].name, "encode");
    assert_eq!(info.methods[1].name, "describe");
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior methods should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}
