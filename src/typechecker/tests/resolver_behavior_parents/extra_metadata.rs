use super::*;

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}

Debug: behavior {
    debug: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_names_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec!["Json".to_string(), "Debug".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior parent metadata should fail");

    let expected =
        "resolver behavior symbol 'PrettyJson' has parents 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior parent metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_refs() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}

Debug: behavior {
    debug: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec![
            BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![],
            },
            BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            },
        ]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior parent ref metadata should fail");

    let expected =
        "resolver behavior symbol 'PrettyJson' has parent refs 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior parent ref diagnostic, got {err:?}"
    );
}
