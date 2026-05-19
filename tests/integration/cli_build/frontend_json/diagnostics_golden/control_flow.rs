use super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_missing_bool_match_arm_schema_matches_golden() {
    assert_diagnostics_golden(
        "missing_bool_match_arm.zen",
        r#"
main = (flag: bool) i32 {
    flag ?
        | true { 1 }
}
"#,
        "tests/fixtures/ir_json/diagnostics_missing_bool_match_arm.golden.json",
        "missing bool match arm",
        1,
        "missing bool match arm diagnostics should carry one structured fix",
    );
}
