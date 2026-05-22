use super::*;

#[test]
fn collect_declarations_with_symbols_avoids_false_duplicate_from_restored_parent_type_args() {
    let mut program = parse_program(
        r#"
Marker<T>: behavior {
}
PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Marker<StaticString>)
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
    assert_eq!(parent_keys, vec!["Marker_StaticString", "Marker_i32"]);
    assert!(
        tc.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("duplicate behavior inheritance")),
        "resolver-restored parent type args should avoid false duplicate diagnostics: {:?}",
        tc.diagnostics
    );
}
