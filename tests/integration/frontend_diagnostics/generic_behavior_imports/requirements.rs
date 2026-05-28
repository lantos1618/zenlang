use super::super::support::assert_diagnostic_code_and_message;
use super::generic_json_diagnostics;

#[test]
fn imported_generic_behavior_requires_missing_impl_is_error() {
    let diagnostics = generic_json_diagnostics(
        r#"
{ Json } = traits

Point: {
    x: i32
}

Point.requires(Json<StaticString>)

main = () i32 {
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E6007",
        "type `Point` does not implement required behavior `Json_StaticString`",
        "imported generic behavior requires",
    );
}
