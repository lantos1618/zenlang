use super::assert_mir_golden;

#[test]
fn emit_json_mir_generic_vec_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_vec.zen",
        "tests/fixtures/ir_json/mir_generic_vec.golden.json",
        "generic Vec input",
    );
}
