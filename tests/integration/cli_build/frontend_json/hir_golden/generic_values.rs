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

#[test]
fn emit_json_hir_generic_worklist_dedup_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_worklist_dedup.zen",
        "tests/fixtures/ir_json/hir_generic_worklist_dedup.golden.json",
        "generic worklist dedup input",
    );
}

#[test]
fn emit_json_hir_generic_ufc_dedup_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_ufc_dedup.zen",
        "tests/fixtures/ir_json/hir_generic_ufc_dedup.golden.json",
        "generic UFC dedup input",
    );
}

#[test]
fn emit_json_hir_generic_ufc_function_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_ufc_function.zen",
        "tests/fixtures/ir_json/hir_generic_ufc_function.golden.json",
        "generic UFC function input",
    );
}
