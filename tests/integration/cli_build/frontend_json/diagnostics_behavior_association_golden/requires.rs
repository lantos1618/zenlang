use super::assert_behavior_association_diagnostics_golden;

#[test]
fn emit_json_diagnostics_generic_requires_missing_impl_schema_matches_golden() {
    assert_behavior_association_diagnostics_golden(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.requires(Json<StaticString>)

main = () i32 {
    0
}
"#,
        "generic_requires_missing_impl.zen",
        "tests/fixtures/ir_json/diagnostics_generic_requires_missing_impl.golden.json",
        "missing generic behavior requires impl",
    );
}

#[test]
fn emit_json_diagnostics_duplicate_generic_requires_schema_matches_golden() {
    assert_behavior_association_diagnostics_golden(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
Point.requires(Json<StaticString>)

main = () i32 {
    0
}
"#,
        "duplicate_generic_requires.zen",
        "tests/fixtures/ir_json/diagnostics_duplicate_generic_requires.golden.json",
        "duplicate generic behavior requires",
    );
}

#[test]
fn emit_json_diagnostics_generic_requires_arity_schema_matches_golden() {
    assert_behavior_association_diagnostics_golden(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.requires(Json<i32, StaticString>)

main = () i32 {
    0
}
"#,
        "generic_requires_arity.zen",
        "tests/fixtures/ir_json/diagnostics_generic_requires_arity.golden.json",
        "generic behavior requires arity",
    );
}
