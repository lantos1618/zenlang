use super::*;

#[test]
fn resolver_rejects_unknown_type_references_in_declarations() {
    let program = parse_program(
        r#"
Point: { next: MissingPoint }
distance = (point: Point) UnknownReturn { 0 }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown type references should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'MissingPoint'")),
        "expected missing field type diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'UnknownReturn'")),
        "expected missing return type diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_method_on_unknown_type() {
    let program = parse_program(
        r#"
Missing.label = () StaticString { "missing" }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("method receiver type should be known");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown method receiver type diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_self_type_outside_method_or_behavior() {
    let program = parse_program(
        r#"
main = (value: Self) i32 { 0 }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("Self should require a method or behavior context");

    assert!(
        err.iter()
            .any(|d| d.message.contains("Self type is only valid")),
        "expected invalid Self type diagnostic, got {err:?}"
    );
}
