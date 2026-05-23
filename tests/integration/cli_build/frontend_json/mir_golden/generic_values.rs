use super::assert_mir_golden;

#[test]
fn emit_json_mir_generic_vec_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_vec.zen",
        "tests/fixtures/ir_json/mir_generic_vec.golden.json",
        "generic Vec input",
    );
}

#[test]
fn emit_json_mir_generic_function_worklist_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_worklist.zen",
        "tests/fixtures/ir_json/mir_generic_function_worklist.golden.json",
        "generic function worklist input",
    );
}

#[test]
fn emit_json_mir_generic_worklist_dedup_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_worklist_dedup.zen",
        "tests/fixtures/ir_json/mir_generic_worklist_dedup.golden.json",
        "generic worklist dedup input",
    );
}

#[test]
fn emit_json_mir_multi_file_generic_imported_worklist_chain_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/multi_file_generic_imported_worklist_chain/main.zen",
        "tests/fixtures/ir_json/mir_multi_file_generic_imported_worklist_chain.golden.json",
        "multi-file imported generic worklist chain input",
    );
}

#[test]
fn emit_json_mir_multi_file_generic_imported_diamond_same_name_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/multi_file_generic_imported_diamond_same_name/main.zen",
        "tests/fixtures/ir_json/mir_multi_file_generic_imported_diamond_same_name.golden.json",
        "multi-file imported generic diamond same-name input",
    );
}

#[test]
fn emit_json_mir_multi_file_imported_generic_function_return_enum_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/multi_file_imported_generic_function_return_enum_dependency/main.zen",
        "tests/fixtures/ir_json/mir_multi_file_imported_generic_function_return_enum.golden.json",
        "multi-file imported generic function return enum input",
    );
}

#[test]
fn emit_json_mir_generic_recursive_function_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_recursive_function.zen",
        "tests/fixtures/ir_json/mir_generic_recursive_function.golden.json",
        "generic recursive function input",
    );
}

#[test]
fn emit_json_mir_generic_ufc_dedup_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_ufc_dedup.zen",
        "tests/fixtures/ir_json/mir_generic_ufc_dedup.golden.json",
        "generic UFC dedup input",
    );
}

#[test]
fn emit_json_mir_generic_ufc_function_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_ufc_function.zen",
        "tests/fixtures/ir_json/mir_generic_ufc_function.golden.json",
        "generic UFC function input",
    );
}
