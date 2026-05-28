use super::assert_mir_golden;

#[test]
fn emit_json_mir_generic_value_schemas_match_golden() {
    for (source, golden_stem, description) in [
        (
            "tests/zen/generic_vec.zen",
            "generic_vec",
            "generic Vec input",
        ),
        (
            "tests/zen/generic_worklist.zen",
            "generic_function_worklist",
            "generic function worklist input",
        ),
        (
            "tests/zen/generic_worklist_dedup.zen",
            "generic_worklist_dedup",
            "generic worklist dedup input",
        ),
        (
            "tests/zen/multi_file_generic_imported_worklist_chain/main.zen",
            "multi_file_generic_imported_worklist_chain",
            "multi-file imported generic worklist chain input",
        ),
        (
            "tests/zen/multi_file_generic_imported_worklist_multi_specialization/main.zen",
            "multi_file_generic_imported_worklist_multi_specialization",
            "multi-file imported generic worklist multi-specialization input",
        ),
        (
            "tests/zen/multi_file_generic_imported_diamond_same_name/main.zen",
            "multi_file_generic_imported_diamond_same_name",
            "multi-file imported generic diamond same-name input",
        ),
        (
            "tests/zen/multi_file_imported_generic_function_return_enum_dependency/main.zen",
            "multi_file_imported_generic_function_return_enum",
            "multi-file imported generic function return enum input",
        ),
        (
            "tests/zen/generic_recursive_function.zen",
            "generic_recursive_function",
            "generic recursive function input",
        ),
        (
            "tests/zen/generic_ufc_dedup.zen",
            "generic_ufc_dedup",
            "generic UFC dedup input",
        ),
        (
            "tests/zen/generic_ufc_function.zen",
            "generic_ufc_function",
            "generic UFC function input",
        ),
    ] {
        assert_mir_golden(source, golden_stem, description);
    }
}
