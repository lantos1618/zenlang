use super::super::support::assert_diagnostic_code_and_message;
use super::generic_json_diagnostics;

#[test]
fn imported_generic_behavior_impl_type_arg_arity_is_error() {
    let diagnostics = generic_json_diagnostics(
        r#"
{ Json } = traits

Point: {
    x: i32
}

Point.implements(Json) {
    encode = (value: Point) StaticString {
        "point"
    }
}

main = () i32 {
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5001",
        "generic behavior `Json` expects 1 type arguments, found 0",
        "imported behavior impl arity",
    );
}

#[test]
fn imported_generic_behavior_extends_type_arg_arity_is_error() {
    let diagnostics = generic_json_diagnostics(
        r#"
{ Json } = traits

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)

main = () i32 {
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5001",
        "generic behavior `Json` expects 1 type arguments, found 0",
        "imported behavior extends arity",
    );
}

#[test]
fn imported_generic_behavior_requires_type_arg_arity_is_error() {
    let diagnostics = generic_json_diagnostics(
        r#"
{ Json } = traits

Point: {
    x: i32
}

Point.requires(Json<i32, StaticString>)

main = () i32 {
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5001",
        "generic behavior `Json` expects 1 type arguments, found 2",
        "imported behavior requires arity",
    );
}

#[test]
fn imported_generic_behavior_bound_type_arg_arity_is_error() {
    let diagnostics = generic_json_diagnostics(
        r#"
{ Json } = traits

encode<T: Json> = (value: T) StaticString {
    "encoded"
}

main = () i32 {
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E5001",
        "generic behavior `Json` expects 1 type arguments, found 0",
        "imported behavior bound arity",
    );
}
