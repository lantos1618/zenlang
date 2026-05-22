use super::*;

#[test]
fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_parent_metadata() {
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
