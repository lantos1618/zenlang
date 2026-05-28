use super::super::support::{
    assert_diagnostic_code_and_message, frontend_diagnostics_for_module,
    frontend_diagnostics_for_modules,
};
use super::{GENERIC_JSON_TRAIT, JSON_PRETTY_TRAITS};

#[test]
fn imported_behavior_extends_requires_parent_methods() {
    let diagnostics = frontend_diagnostics_for_module(
        "traits.zen",
        JSON_PRETTY_TRAITS,
        r#"
{ PrettyJson } = traits

Point: {
    x: i32
}

Point.implements(PrettyJson) {
    pretty = (value: Point) StaticString {
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
        "E6001",
        "implementation of `PrettyJson` is missing required method `encode`",
        "imported inherited behavior method",
    );
}

#[test]
fn imported_behavior_extends_imported_parent_requires_parent_methods() {
    let diagnostics = frontend_diagnostics_for_modules(
        &[
            ("base.zen", GENERIC_JSON_TRAIT),
            (
                "traits.zen",
                r#"
{ Json } = base

pub PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)
"#,
            ),
        ],
        r#"
{ PrettyJson } = traits

Point: {
    x: i32
}

Point.implements(PrettyJson) {
    pretty = (value: Point) StaticString {
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
        "E6001",
        "implementation of `PrettyJson` is missing required method `encode`",
        "imported parent behavior method",
    );
}

#[test]
fn imported_behavior_extends_requires_transitive_parent_methods() {
    let diagnostics = frontend_diagnostics_for_module(
        "traits.zen",
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}

pub PrettyJson: behavior {
    pretty: (Self) StaticString
}

pub FancyJson: behavior {
    fancy: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)
FancyJson.extends(PrettyJson)
"#,
        r#"
{ FancyJson } = traits

Point: {
    x: i32
}

Point.implements(FancyJson) {
    pretty = (value: Point) StaticString {
        "pretty"
    }

    fancy = (value: Point) StaticString {
        "fancy"
    }
}

main = () i32 {
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E6001",
        "implementation of `FancyJson` is missing required method `encode`",
        "transitive inherited behavior method",
    );
}
