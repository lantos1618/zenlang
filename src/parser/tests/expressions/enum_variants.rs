use super::*;

#[test]
fn parse_enum_variant_expr() {
    let prog = parse_ok("f = () void {\n    s = Status.Active\n}");
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_enum_variant_payload_expr() {
    let prog = parse_ok("f = () void {\n    s = Maybe.Some(42)\n}");
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_shorthand_enum_variant_expr_and_pattern() {
    let prog = parse_ok(
        r#"f = (value: Result<i32, StaticString>) Result<i32, StaticString> {
    value ?
        | .Ok(v) { .Ok(v) }
        | .Err(msg) { .Err(msg) }
}"#,
    );
    assert_eq!(prog.declarations.len(), 1);
}
