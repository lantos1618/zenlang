use super::super::support::assert_diagnostic_code_and_message;
use super::generic_json_diagnostics;

#[test]
fn imported_duplicate_generic_behavior_requires_is_error() {
    let diagnostics = generic_json_diagnostics(
        r#"
{ Json } = traits

Point: {
    x: i32
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString {
        "point"
    }
}

Point.requires(Json<StaticString>)
Point.requires(Json<StaticString>)

main = () i32 {
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E3500",
        "duplicate required behavior `Json<StaticString>` for `Point`",
        "imported duplicate behavior requires",
    );
}
