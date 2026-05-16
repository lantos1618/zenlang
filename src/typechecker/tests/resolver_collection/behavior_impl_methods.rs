use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_behavior_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}
Debug: behavior {
    describe: (Self) str
}

Point.implements(Json) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { behavior, .. } = &mut program.declarations[3] {
        *behavior = Some("Debug".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let method = tc
        .methods
        .get("Point.encode")
        .expect("resolver-restored behavior default");
    assert_eq!(method.params[0].0, "self");
    assert_eq!(method.return_type, AstType::Str);
    assert!(
        !tc.methods.contains_key("Point.describe"),
        "stale AST-only behavior default should not be synthesized"
    );
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl metadata should drive default synthesis: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}

Point.implements(Json) {
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
        "resolver-restored behavior impl target should drive omitted default synthesis: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_and_name_for_defaults() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
}
Debug: behavior {
    describe: (Self) str
}

Point.implements(Json) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        ..
    } = &mut program.declarations[3]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("Debug".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(tc.methods.contains_key("Point.encode"));
    assert!(!tc.methods.contains_key("Missing.encode"));
    assert!(
        !tc.methods.contains_key("Point.describe"),
        "stale AST-only behavior default should not be synthesized"
    );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior impl target and name should drive omitted default synthesis: {:?}",
            tc.diagnostics
        );
}

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
fn collect_declarations_with_symbols_uses_resolver_impl_method_metadata_for_impl_checks() {
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
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            params[1].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl method metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_impl_method_name_metadata_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { name, .. } = &mut methods[0] {
            *name = "missing".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl method name metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_let_stale_ast_name_hide_extra_impl_method() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    extra = (value: Point) str { "extra" }
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
fn collect_declarations_with_symbols_uses_resolver_impl_method_parameter_names_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (value: Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params[0].name = "stale".to_string();
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.encode").expect("impl method info");
    assert_eq!(info.params[0].0, "value");
    assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method parameter names should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_impl_method_parameter_order_for_impl_checks() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (value: Self, input: i32) str
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function { params, .. } = &mut methods[0] {
            params.swap(0, 1);
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.map").expect("impl method info");
    assert_eq!(info.params[0].0, "value");
    assert_eq!(info.params[1].0, "input");
    assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
    assert_eq!(info.params[1].1, AstType::I32);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method parameter order should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_name_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
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
    encode: (Self) str
}

Point.impl = {
    get = (self: Point) i32 { self.x }
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
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

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_impl_method_signature()
{
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
    if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should not keep AST-only method metadata when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_behavior_impl_method_signature_after_key_restore()
{
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        !tc.methods.contains_key("Missing.missing"),
        "resolver-backed behavior impl collection should not keep stale AST method keys"
    );
    assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should clear restored method keys when resolver signature metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_method_signature_target_and_name_metadata(
) {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.methods.get("Point.encode").expect("impl method info");
    assert!(!tc.methods.contains_key("Missing.missing"));
    assert_eq!(info.params[0].0, "value");
    assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
    assert_eq!(info.return_type, AstType::Str);
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl method signature should avoid stale AST diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata(
) {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode<T> = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params.pop();
            *return_type = None;
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let template = tc
        .generic_methods
        .get("Point.encode")
        .expect("generic behavior impl method template");
    assert!(!tc.generic_methods.contains_key("Missing.missing"));
    assert!(!tc.generic_methods.contains_key("Point.missing"));
    assert_eq!(template.type_params, vec!["T".to_string()]);
    assert_eq!(template.params.len(), 1);
    assert_eq!(template.params[0].name, "value");
    assert_eq!(template.params[0].ty, AstType::Named("Point".to_string()));
    assert_eq!(template.return_type, Some(AstType::Str));
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior impl generic template should avoid stale AST diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_clears_stale_behavior_impl_generic_method_template_after_key_restore(
) {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode<T> = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
    if let Declaration::ImplBlock {
        type_name, methods, ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut methods[0]
        {
            *name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        !tc.generic_methods.contains_key("Missing.missing"),
        "resolver-backed behavior impl collection should clear stale AST generic method templates"
    );
    assert!(
            !tc.generic_methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should clear restored generic method templates when resolver signature metadata is incomplete"
        );
}
