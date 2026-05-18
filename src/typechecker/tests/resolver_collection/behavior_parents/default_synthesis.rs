use super::*;

#[test]
fn collect_declarations_with_symbols_synthesizes_defaults_from_restored_behavior_parent() {
    let mut program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString { "json" }
}
PrettyJson: behavior {
    pretty: (Self) StaticString
}
Point: { x: i32 }

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    pretty = (value: Point) StaticString { "point" }
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
    pretty: (Self) StaticString
}
Point: { x: i32 }

PrettyJson.extends(Json<StaticString>)

Point.implements(PrettyJson) {
    pretty = (value: Point) StaticString { "point" }
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
