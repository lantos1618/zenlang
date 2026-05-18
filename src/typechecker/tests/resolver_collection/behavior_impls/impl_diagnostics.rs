use super::*;

#[test]
fn collect_declarations_with_symbols_reports_resolver_restored_impl_target_and_name() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<StaticString>) {
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
                *message == "type `Point` implementation of `Json_StaticString` is missing required method `encode`"
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
    pretty: (Self) StaticString
}
Point: { x: i32 }

PrettyJson.extends(Json<StaticString>)

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(PrettyJson) {
    pretty = (value: Point) StaticString { "point" }
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
                    == "overlapping implementations of behaviors `Json_StaticString` and `PrettyJson` for type `Point`"
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

Point.implements(Json<StaticString>) {
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
            .contains(&("Point".to_string(), "Json_StaticString".to_string()))
            && tc
                .behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
        "resolver-restored impl type args should keep distinct impl specializations: {:?}",
        tc.behavior_impls
    );
}
