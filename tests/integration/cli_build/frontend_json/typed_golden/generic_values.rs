use super::assert_typed_golden;

#[test]
fn emit_json_typed_generic_ufc_dedup_schema_matches_golden() {
    assert_typed_golden(
        "generic_ufc_dedup.zen",
        "tests/fixtures/ir_json/typed_generic_ufc_dedup.golden.json",
        "generic UFC dedup",
    );
}

#[test]
fn emit_json_typed_generic_ufc_function_schema_matches_golden() {
    assert_typed_golden(
        "generic_ufc_function.zen",
        "tests/fixtures/ir_json/typed_generic_ufc_function.golden.json",
        "generic UFC function",
    );
}

#[test]
fn emit_json_typed_generic_worklist_dedup_schema_matches_golden() {
    assert_typed_golden(
        "generic_worklist_dedup.zen",
        "tests/fixtures/ir_json/typed_generic_worklist_dedup.golden.json",
        "generic worklist dedup",
    );
}

#[test]
fn emit_json_typed_generic_recursive_function_schema_matches_golden() {
    assert_typed_golden(
        "generic_recursive_function.zen",
        "tests/fixtures/ir_json/typed_generic_recursive_function.golden.json",
        "generic recursive function",
    );
}

#[test]
fn emit_json_typed_multi_file_imported_generic_function_return_enum_schema_matches_golden() {
    assert_typed_golden(
        "multi_file_imported_generic_function_return_enum_dependency/main.zen",
        "tests/fixtures/ir_json/typed_multi_file_imported_generic_function_return_enum.golden.json",
        "multi-file imported generic function return enum",
    );
}

#[test]
fn emit_json_typed_generic_vec_schema_matches_golden() {
    assert_typed_golden(
        "generic_vec.zen",
        "tests/fixtures/ir_json/typed_generic_vec.golden.json",
        "generic Vec",
    );
}
