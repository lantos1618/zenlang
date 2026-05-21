use super::*;

mod default_synthesis;
mod method_metadata;
mod restored_signatures;

#[test]
fn collect_declarations_with_symbols_defers_impl_checks_until_resolver_metadata_is_collected() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32
}

Point.implements(Mapper) {
    map = (self: Point, callback: (i32) i32) (i32) i32 { callback }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params[1].ty = AstType::I32;
        methods[0].return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_let_stale_ast_name_hide_extra_impl_method() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    extra = (value: Point) StaticString { "extra" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { name, .. } = &mut methods[0] {
            *name = "encode".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| *message == "method `extra` is not declared by behavior `Json`"),
        "resolver-owned extra impl method should not be hidden by stale AST required name: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_name_metadata() {
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
    if let Declaration::ImplBlock { type_name, .. } = &mut program.declarations[2] {
        *type_name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.encode"));
    assert!(!tc.methods.contains_key("Missing.encode"));
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl target should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn resolver_declaration_metadata_skips_behavior_impl_methods_until_behavior_impl_pass() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) StaticString
}

Point.impl = {
    get = (self: Point) i32 { self.x }
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let mut tc = TypeChecker::new();

    tc.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&program.declarations);
    });
    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    tc.collect_resolver_declaration_metadata(&symbols, &tasks);

    assert!(
        tc.methods.contains_key("Point.get"),
        "non-behavior impl methods should still be refreshed by declaration metadata"
    );
    assert!(
        !tc.methods.contains_key("Point.encode"),
        "behavior impl method signatures should be owned by the behavior impl metadata pass"
    );
}
