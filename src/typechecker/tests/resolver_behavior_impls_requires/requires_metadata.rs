use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_behavior_required_names() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_names_for_test(Namespace::Type, "Point", None);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior requires metadata mismatch should fail");

    let expected =
        "resolver type symbol 'Point' has behavior requires 'none', expected to include 'Json'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver behavior requires metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_required_names() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec!["Json<i32>".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior requires metadata mismatch should fail");

    let expected =
            "resolver type symbol 'Point' has behavior requires 'Json<i32>', expected to include 'Json<StaticString>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior requires metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_behavior_required_refs() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_required_refs_for_test(
        Namespace::Type,
        "Point",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic behavior requires ref mismatch should fail");

    let expected =
            "resolver type symbol 'Point' has behavior requires refs 'Json<i32>', expected to include 'Json<StaticString>'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver generic behavior requires ref diagnostic, got {err:?}"
    );
}
