use super::*;

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_impl_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec!["Json".to_string(), "Debug".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior impl metadata should fail");

    let expected = "resolver type symbol 'Point' has behavior impls 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior impl metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_impl_refs() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Type,
        "Point",
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
        .expect_err("extra resolver behavior impl ref metadata should fail");

    let expected =
        "resolver type symbol 'Point' has behavior impl refs 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior impl ref diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_required_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec!["Json".to_string(), "Debug".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("extra resolver behavior requires metadata should fail");

    let expected =
        "resolver type symbol 'Point' has behavior requires 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior requires metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_rejects_extra_resolver_behavior_required_refs() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_refs_for_test(
        Namespace::Type,
        "Point",
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
        .expect_err("extra resolver behavior requires ref metadata should fail");

    let expected =
        "resolver type symbol 'Point' has behavior requires refs 'Json, Debug', expected 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected extra resolver behavior requires ref diagnostic, got {err:?}"
    );
}
