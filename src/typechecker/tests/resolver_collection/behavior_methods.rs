use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_metadata() {
    let mut program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[0] {
        methods[0].params[1].ty = AstType::I32;
        methods[0].return_type = Some(AstType::I32);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Mapper").expect("behavior info");
    assert_eq!(
        info.methods[0].params[1].ty,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
    assert_eq!(
        info.methods[0].return_type,
        Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        })
    );
}

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

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_name_metadata() {
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
fn collect_declarations_with_symbols_uses_resolver_behavior_method_parameter_count() {
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
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params.push(Param {
            name: "stale".to_string(),
            ty: AstType::I32,
            mutable: false,
            span: Span::dummy(),
        });
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].params.len(), 1);
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior method params should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_missing_parameter_count() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (Self, i32) str
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params.pop();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Mapper").expect("behavior info");
    assert_eq!(info.methods[0].params.len(), 2);
    assert_eq!(info.methods[0].params[0].name, "__arg0");
    assert_eq!(info.methods[0].params[1].name, "__arg1");
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert_eq!(info.methods[0].params[1].ty, AstType::I32);
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored behavior method params should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_parameter_names() {
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
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params[0].name = "stale".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Json").expect("behavior info");
    assert_eq!(info.methods[0].params[0].name, "value");
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method parameter names should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_ignores_stale_behavior_method_parameter_order() {
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
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].params.swap(0, 1);
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc.behaviors.get("Mapper").expect("behavior info");
    assert_eq!(info.methods[0].params[0].name, "value");
    assert_eq!(info.methods[0].params[1].name, "input");
    assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
    assert_eq!(info.methods[0].params[1].ty, AstType::I32);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method parameter order should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_method_count() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
    describe: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
    describe = (value: Point) str { "desc" }
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

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_default_method_metadata() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32 { callback }
}

Point.implements(Mapper) {
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

    let info = tc.methods.get("Point.map").expect("default method info");
    assert_eq!(
        info.params[1].1,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
    assert_eq!(
        info.return_type,
        AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::I32),
        }
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_default_method_name_metadata() {
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
    if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
        methods[0].name = "missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let info = tc
        .methods
        .get("Point.encode")
        .expect("resolver-restored default method");
    assert_eq!(info.params[0].0, "self");
    assert_eq!(info.return_type, AstType::Str);
    assert!(
        !tc.methods.contains_key("Point.missing"),
        "stale AST-only behavior default method name should not be synthesized"
    );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior default method name should drive omitted default synthesis: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_skips_default_when_resolver_restores_impl_method_name() {
    let mut program = parse_program(
        r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { "default" }
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

    let method = tc
        .methods
        .get("Point.encode")
        .expect("restored impl method");
    assert_eq!(
        method.params[0].0, "value",
        "resolver-restored explicit impl method should not be overwritten by the behavior default"
    );
    assert!(
        !tc.methods.contains_key("Point.missing"),
        "stale AST-only impl method key should be removed"
    );
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl method name should suppress default insertion: {:?}",
        tc.diagnostics
    );
}
