use super::assert_behavior_association_diagnostics_golden;

#[test]
fn emit_json_diagnostics_generic_impl_arity_schema_matches_golden() {
    assert_behavior_association_diagnostics_golden(
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
        "tests/fixtures/ir_json/diagnostics_generic_impl_arity.golden.json",
        "generic behavior impl arity",
    );
}

#[test]
fn emit_json_diagnostics_generic_extends_arity_schema_matches_golden() {
    assert_behavior_association_diagnostics_golden(
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
        "tests/fixtures/ir_json/diagnostics_generic_extends_arity.golden.json",
        "generic behavior extends arity",
    );
}

#[test]
fn emit_json_diagnostics_duplicate_generic_impl_schema_matches_golden() {
    assert_behavior_association_diagnostics_golden(
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
        "tests/fixtures/ir_json/diagnostics_duplicate_generic_impl.golden.json",
        "duplicate generic behavior impl",
    );
}
