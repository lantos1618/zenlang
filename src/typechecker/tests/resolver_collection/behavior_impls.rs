use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = &mut program.declarations[2]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string())),
        "resolver metadata should restore the validated Json<str> impl"
    );
    assert!(
        !tc.behavior_impls
            .contains(&("Point".to_string(), "Json_i32".to_string())),
        "AST-only Json<i32> impl drift should not remain after resolver collection"
    );
    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_impl_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = &mut program.declarations[2]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
            "resolver-backed collection should not keep AST-only behavior impl refs when resolver impl metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_synthesize_stale_impl_defaults_after_target_restore()
{
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
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("AlsoMissing".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behavior_impls
                .contains(&("Point".to_string(), "Json".to_string())),
            "resolver-backed collection should not keep AST-only behavior impl refs when resolver impl metadata is incomplete"
        );
    assert!(
        !tc.methods.contains_key("Missing.encode"),
        "resolver-backed default synthesis should not keep stale AST target method keys"
    );
    assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed default synthesis should not synthesize behavior defaults when resolver impl metadata is incomplete"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only impl refs after target restoration when resolver impl metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock { behavior, .. } = &mut program.declarations[2] {
        *behavior = Some("Missing".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored impl name metadata should avoid stale AST impl diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_and_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("AlsoMissing".to_string());
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string())),
        "resolver metadata should restore the validated Point implements Json<str> association"
    );
    assert!(
            !tc.behavior_impls
                .contains(&("Missing".to_string(), "AlsoMissing_i32".to_string())),
            "stale AST-only impl target and behavior metadata should not remain after resolver collection"
        );
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl target and behavior metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_reports_resolver_restored_impl_target_and_name() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::ImplBlock {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = Some("AlsoMissing".to_string());
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
            messages.iter().any(|message| {
                *message == "type `Point` implementation of `Json_str` is missing required method `encode`"
            }),
            "resolver-restored impl metadata should report the validated missing method, got {:?}",
            messages
        );
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("Missing") && !message.contains("AlsoMissing")),
        "stale AST-only impl names should not leak into diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_reports_overlap_from_restored_impl_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let first_impl = program
        .declarations
        .iter_mut()
        .find(|declaration| {
            matches!(
                declaration,
                Declaration::ImplBlock {
                    behavior: Some(behavior),
                    ..
                } if behavior == "Json"
            )
        })
        .expect("Json impl declaration");
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = first_impl
    {
        behavior_type_args[0] = AstType::I32;
    } else {
        panic!("expected Json impl declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let messages: Vec<_> = tc
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert!(
            messages.iter().any(|message| {
                *message
                    == "overlapping implementations of behaviors `Json_str` and `PrettyJson` for type `Point`"
            }),
            "resolver-restored impl type args should drive overlap diagnostics, got {:?}",
            messages
        );
    assert!(
        messages.iter().all(|message| !message.contains("Json_i32")),
        "stale AST-only impl type args should not leak into overlap diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_avoids_false_duplicate_from_restored_impl_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T { "default" }
}
Point: { x: i32 }

Point.implements(Json<str>) {
}

Point.implements(Json<i32>) {
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_impl = program
        .declarations
        .iter_mut()
        .filter(|declaration| {
            matches!(
                declaration,
                Declaration::ImplBlock {
                    behavior: Some(behavior),
                    ..
                } if behavior == "Json"
            )
        })
        .nth(1)
        .expect("second Json impl declaration");
    if let Declaration::ImplBlock {
        behavior_type_args, ..
    } = second_impl
    {
        behavior_type_args[0] = AstType::Str;
    } else {
        panic!("expected second Json impl declaration");
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
            .all(|message| !message.contains("duplicate implementation")),
        "resolver-restored impl type args should avoid false duplicate diagnostics, got {:?}",
        messages
    );
    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string()))
            && tc
                .behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
        "resolver-restored impl type args should keep distinct impl specializations: {:?}",
        tc.behavior_impls
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires {
        behavior_type_args, ..
    } = &mut program.declarations[3]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored requires metadata should avoid stale AST requires diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_target_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires { type_name, .. } = &mut program.declarations[3] {
        *type_name = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires target metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_target_and_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *type_name = "Missing".to_string();
        *behavior = "AlsoMissing".to_string();
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires target and behavior metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_reports_resolver_restored_required_target_and_name() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *type_name = "Missing".to_string();
        *behavior = "AlsoMissing".to_string();
        behavior_type_args[0] = AstType::I32;
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
            .any(|message| *message
                == "type `Point` does not implement required behavior `Json_str`"),
        "resolver-restored requires metadata should report the validated missing impl, got {:?}",
        messages
    );
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("Missing") && !message.contains("AlsoMissing")),
        "stale AST-only requires names should not leak into diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_uses_restored_requires_ref_for_inherited_impl() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { "point" }
    pretty = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let requires = program
        .declarations
        .iter_mut()
        .find(|declaration| matches!(declaration, Declaration::Requires { .. }))
        .expect("requires declaration");
    if let Declaration::Requires {
        behavior,
        behavior_type_args,
        ..
    } = requires
    {
        *behavior = "Missing".to_string();
        behavior_type_args[0] = AstType::I32;
    } else {
        panic!("expected requires declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.is_empty(),
        "resolver-restored requires ref should be satisfied by inherited child impl: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_distinct_restored_requires_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T { "default" }
}
Point: { x: i32 }

Point.implements(Json<str>) {
}

Point.implements(Json<i32>) {
}

Point.requires(Json<str>)
Point.requires(Json<i32>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_requires = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::Requires { .. }))
        .nth(1)
        .expect("second requires declaration");
    if let Declaration::Requires {
        behavior_type_args, ..
    } = second_requires
    {
        behavior_type_args[0] = AstType::Str;
    } else {
        panic!("expected requires declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("does not implement required behavior")),
        "resolver-restored requires type args should keep distinct satisfied specializations: {:?}",
        tc.diagnostics
    );
    assert!(
        tc.behavior_impls
            .contains(&("Point".to_string(), "Json_str".to_string()))
            && tc
                .behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
        "resolver-restored impl refs should keep both required specializations available: {:?}",
        tc.behavior_impls
    );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_required_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::Requires {
        behavior_type_args, ..
    } = &mut program.declarations[3]
    {
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only requires refs when resolver required metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_validate_stale_requires_after_target_restore() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_refs_for_test(Namespace::Type, "Point", None);
    if let Declaration::Requires {
        type_name,
        behavior,
        behavior_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *type_name = "Missing".to_string();
        *behavior = "AlsoMissing".to_string();
        behavior_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only requires refs after target restoration when resolver required metadata is incomplete: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_required_name_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Requires { behavior, .. } = &mut program.declarations[3] {
        *behavior = "Missing".to_string();
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires name metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
}
