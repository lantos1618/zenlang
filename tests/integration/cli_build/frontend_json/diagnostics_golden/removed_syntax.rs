use super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_removed_return_schema_matches_golden() {
    assert_diagnostics_golden(
        "return_keyword.zen",
        r#"
main = () i32 {
    return 1
}
"#,
        "tests/fixtures/ir_json/diagnostics_return.golden.json",
        "removed return syntax",
        1,
        "removed return diagnostics should emit one removed-syntax diagnostic",
    );
}
