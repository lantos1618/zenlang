use super::assert_hir_golden;

#[test]
fn emit_json_hir_generic_vec_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_vec.zen",
        "tests/fixtures/ir_json/hir_generic_vec.golden.json",
        "generic Vec input",
    );
}

#[test]
fn emit_json_hir_generic_function_worklist_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_worklist.zen",
        "tests/fixtures/ir_json/hir_generic_function_worklist.golden.json",
        "generic function worklist input",
    );
}
