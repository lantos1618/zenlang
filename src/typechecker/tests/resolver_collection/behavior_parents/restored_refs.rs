use super::*;

#[test]
fn collect_declarations_with_symbols_uses_resolver_behavior_parent_metadata() {
    let mut program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)
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
