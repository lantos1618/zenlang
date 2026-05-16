use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_parent_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[2]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    assert_eq!(parents[0].behavior, "Json");
    assert_eq!(parents[0].type_args, vec![AstType::Str]);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior parent metadata should avoid stale AST extends diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_parent_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_refs_for_test(Namespace::Behavior, "PrettyJson", None);
    if let Declaration::BehaviorExtends {
        parent_type_args, ..
    } = &mut program.declarations[2]
    {
        parent_type_args[0] = AstType::I32;
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
            !tc.behavior_extends.contains_key("PrettyJson"),
            "resolver-backed collection should not keep AST-only behavior parent refs when resolver parent metadata is incomplete"
        );
}

#[test]
fn collect_declarations_with_symbols_avoids_false_duplicate_from_restored_parent_type_args() {
    let mut program = parse_program(
        r#"
Marker<T>: behavior {
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Marker<str>)
PrettyJson.extends(Marker<i32>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_parent = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
        .nth(1)
        .expect("second parent declaration");
    if let Declaration::BehaviorExtends {
        parent_type_args, ..
    } = second_parent
    {
        parent_type_args[0] = AstType::Str;
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    let parent_keys: Vec<_> = parents.iter().map(|parent| parent.key.as_str()).collect();
    assert_eq!(parent_keys, vec!["Marker_str", "Marker_i32"]);
    assert!(
        tc.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("duplicate behavior inheritance")),
        "resolver-restored parent type args should avoid false duplicate diagnostics: {:?}",
        tc.diagnostics
    );
}

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_parent_and_type_param_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::Behavior { type_params, .. } = &mut program.declarations[2] {
        type_params[0].name = "Stale".to_string();
        type_params[0].constraint = Some("Missing".to_string());
        type_params[0].constraint_type_args.clear();
    }
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::Named("Stale".to_string());
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let parents = tc.behavior_extends.get("Pretty").expect("behavior parents");
    assert_eq!(parents[0].behavior, "Serializable");
    assert_eq!(parents[0].type_args, vec![AstType::Named("T".to_string())]);
    assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior parent and type-parameter metadata should avoid stale AST extends diagnostics: {:?}",
            tc.diagnostics
        );
}

#[test]
fn collect_declarations_with_symbols_reports_resolver_restored_behavior_parent_metadata() {
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
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::I32;
    } else {
        panic!("expected behavior extends declaration");
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
                *message == "type `Point` implementation of `PrettyJson` is missing required method `encode`"
            }),
            "resolver-restored parent metadata should report the inherited missing method, got {:?}",
            messages
        );
    assert!(
        messages.iter().all(|message| !message.contains("Missing")),
        "stale AST-only behavior parent names should not leak into diagnostics: {:?}",
        messages
    );
}

#[test]
fn collect_declarations_with_symbols_reports_conflict_from_restored_parent_type_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
PrettyJson.extends(Debug<i32>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_parent = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
        .nth(1)
        .expect("second parent declaration");
    if let Declaration::BehaviorExtends {
        parent_type_args, ..
    } = second_parent
    {
        parent_type_args[0] = AstType::Str;
    } else {
        panic!("expected behavior extends declaration");
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
                *message == "conflicting behavior method `encode` inherited by `PrettyJson`"
            }),
            "resolver-restored parent type args should drive inherited method coherence diagnostics, got {:?}",
            messages
        );
    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    let parent_keys: Vec<_> = parents.iter().map(|parent| parent.key.as_str()).collect();
    assert_eq!(parent_keys, vec!["Json_str", "Debug_i32"]);
}

#[test]
fn collect_declarations_with_symbols_reports_cycle_from_restored_parent_refs() {
    let mut program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
PrettyJson: behavior {
    pretty: (Self) str
}
Debug: behavior {
    debug: (Self) str
}

Json.extends(PrettyJson)
PrettyJson.extends(Json)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let second_parent = program
        .declarations
        .iter_mut()
        .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
        .nth(1)
        .expect("second parent declaration");
    if let Declaration::BehaviorExtends { parent, .. } = second_parent {
        *parent = "Debug".to_string();
    } else {
        panic!("expected behavior extends declaration");
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
            .any(|message| message.contains("behavior inheritance cycle")),
        "resolver-restored parent refs should drive cycle diagnostics, got {:?}",
        messages
    );
    let parents = tc
        .behavior_extends
        .get("PrettyJson")
        .expect("behavior parents");
    assert_eq!(parents[0].behavior, "Json");
}

#[test]
fn collect_declarations_with_symbols_synthesizes_defaults_from_restored_behavior_parent() {
    let mut program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str { "json" }
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends { parent, .. } = &mut program.declarations[3] {
        *parent = "Missing".to_string();
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    assert!(
        tc.methods.contains_key("Point.encode"),
        "resolver-restored parent metadata should synthesize inherited default method"
    );
    assert!(
        !tc.methods.contains_key("Point.Missing"),
        "stale AST-only parent names should not synthesize default methods"
    );
}

#[test]
fn collect_declarations_with_symbols_synthesizes_generic_defaults_from_restored_parent_args() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T { "json" }
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "point" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    if let Declaration::BehaviorExtends {
        parent,
        parent_type_args,
        ..
    } = &mut program.declarations[3]
    {
        *parent = "Missing".to_string();
        parent_type_args[0] = AstType::I32;
    } else {
        panic!("expected behavior extends declaration");
    }
    let mut tc = TypeChecker::new();

    tc.collect_declarations_with_symbols(&program.declarations, &symbols);

    let encode = tc
        .methods
        .get("Point.encode")
        .expect("resolver-restored parent should synthesize inherited default");
    assert_eq!(
        encode.return_type,
        AstType::Str,
        "resolver-restored parent type args should drive inherited default return type"
    );
}
