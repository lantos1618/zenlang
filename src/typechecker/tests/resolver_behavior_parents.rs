use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_behavior_parent_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_parent_names_for_test(Namespace::Behavior, "PrettyJson", None);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior parent metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver behavior symbol 'PrettyJson' has parents 'none', expected to include 'Json'"
        )),
        "expected resolver behavior parent metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_parent_names() {
    let program = parse_program(
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
    symbols.set_behavior_parent_names_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec!["Json<i32>".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior parent metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'PrettyJson' has parents 'Json<i32>', expected to include 'Json<str>'"
            )),
            "expected resolver generic behavior parent metadata diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_parent_refs() {
    let program = parse_program(
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
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Behavior,
        "PrettyJson",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior parent ref mismatch should fail");

    let expected =
            "resolver behavior symbol 'PrettyJson' has parent refs 'Json<i32>', expected to include 'Json<str>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior parent ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_accepts_resolver_behavior_parent_child_type_param_refs() {
    let program = parse_program(
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
    let mut tc = TypeChecker::new();

    tc.check_program_with_symbols(&program, &symbols)
        .expect("resolver parent type arg using child type parameter should validate");
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
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
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
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
