use super::super::support::{assert_diagnostic_code_and_message, frontend_diagnostics_for_module};
use super::{GENERIC_JSON_TRAIT, JSON_PRETTY_TRAITS};

#[test]
fn imported_behavior_extends_parent_impl_overlap_is_error() {
    let diagnostics = frontend_diagnostics_for_module(
        "traits.zen",
        JSON_PRETTY_TRAITS,
        r#"
{ Json, PrettyJson } = traits

Point: {
    x: i32
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString {
        "point"
    }
}

Point.implements(PrettyJson) {
    encode = (value: Point) StaticString {
        "point"
    }

    pretty = (value: Point) StaticString {
        "pretty"
    }
}

main = () i32 {
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E6010",
        "overlapping implementations of behaviors `Json_StaticString` and `PrettyJson` for type `Point`",
        "imported behavior impl overlap",
    );
}

#[test]
fn imported_behavior_extends_transitive_parent_impl_overlap_is_error() {
    let diagnostics = frontend_diagnostics_for_module(
        "traits.zen",
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}

pub CompactJson: behavior {
    compact: (Self) StaticString
}

pub PrettyJson: behavior {
    pretty: (Self) StaticString
}

CompactJson.extends(Json<StaticString>)
PrettyJson.extends(CompactJson)
"#,
        r#"
{ Json, PrettyJson } = traits

Point: {
    x: i32
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString {
        "point"
    }
}

Point.implements(PrettyJson) {
    encode = (value: Point) StaticString {
        "point"
    }

    compact = (value: Point) StaticString {
        "compact"
    }

    pretty = (value: Point) StaticString {
        "pretty"
    }
}

main = () i32 {
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E6010",
        "overlapping implementations of behaviors `Json_StaticString` and `PrettyJson` for type `Point`",
        "imported transitive behavior impl overlap",
    );
}

#[test]
fn imported_duplicate_generic_behavior_impl_is_error() {
    let diagnostics = frontend_diagnostics_for_module(
        "traits.zen",
        GENERIC_JSON_TRAIT,
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

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString {
        "point-again"
    }
}

main = () i32 {
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E3500",
        "duplicate behavior implementation `Json<StaticString>` for `Point`",
        "imported duplicate behavior impl",
    );
}
