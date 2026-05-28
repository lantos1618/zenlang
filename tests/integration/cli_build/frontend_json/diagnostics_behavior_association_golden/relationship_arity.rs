use super::assert_behavior_association_diagnostics_golden;

#[test]
fn emit_json_diagnostics_generic_relationship_schemas_match_golden() {
    for (source, filename, description) in [
        (
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}

main = () i32 {
    0
}
"#,
            "generic_impl_arity.zen",
            "generic behavior impl arity",
        ),
        (
            r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)

main = () i32 {
    0
}
"#,
            "generic_extends_arity.zen",
            "generic behavior extends arity",
        ),
        (
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point-again" }
}

main = () i32 {
    0
}
"#,
            "duplicate_generic_impl.zen",
            "duplicate generic behavior impl",
        ),
    ] {
        assert_behavior_association_diagnostics_golden(source, filename, description);
    }
}
