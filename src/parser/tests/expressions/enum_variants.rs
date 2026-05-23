use super::*;

#[test]
fn parse_enum_variant_expr() {
    parse_single_decl("f = () void {\n    s = Status.Active\n}");
}

#[test]
fn parse_enum_variant_payload_expr() {
    parse_single_decl("f = () void {\n    s = Maybe.Some(42)\n}");
}

#[test]
fn parse_shorthand_enum_variant_expr_and_pattern() {
    parse_single_decl(
        r#"f = (value: Result<i32, StaticString>) Result<i32, StaticString> {
    value ?
        | .Ok(v) { .Ok(v) }
        | .Err(msg) { .Err(msg) }
}"#,
    );
}
