use super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_unknown_string_type_schema_matches_golden() {
    assert_diagnostics_golden(
        "unknown_string_type.zen",
        r#"
main = (value: String) void { }
"#,
        "unknown String type",
        1,
        "unknown String should emit one resolver diagnostic",
    );
}
