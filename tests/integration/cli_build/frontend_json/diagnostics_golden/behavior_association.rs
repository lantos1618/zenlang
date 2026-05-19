use super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_behavior_derive_gate_schema_matches_golden() {
    assert_diagnostics_golden(
        "derive_gate.zen",
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.derive(Json)
"#,
        "tests/fixtures/ir_json/diagnostics_behavior_derive_gate.golden.json",
        "gated derive association",
        1,
        "gated derive diagnostics should emit one feature-gate diagnostic",
    );
}

#[test]
fn emit_json_diagnostics_generic_association_gate_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_association_gate.zen",
        r#"
Box<T>: {
    value: T
}

Json<T>: behavior {
    to_json: (T) StaticString
}

Box<T>.derive(Json<T>)
"#,
        "tests/fixtures/ir_json/diagnostics_generic_association_gate.golden.json",
        "gated generic association",
        1,
        "gated generic association diagnostics should emit one feature-gate diagnostic",
    );
}

#[test]
fn emit_json_diagnostics_generic_behavior_overlap_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_behavior_overlap.zen",
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.implements(PrettyJson) {
    encode = (value: Point) StaticString { "point" }
    pretty = (value: Point) StaticString { "pretty" }
}
"#,
        "tests/fixtures/ir_json/diagnostics_generic_behavior_overlap.golden.json",
        "overlapping generic behavior impls",
        1,
        "generic behavior overlap should emit one coherence diagnostic",
    );
}
