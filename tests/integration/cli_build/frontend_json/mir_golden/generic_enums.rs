use super::assert_mir_golden;

#[test]
fn emit_json_mir_generic_result_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_result_enum.zen",
        "tests/fixtures/ir_json/mir_generic_result.golden.json",
        "generic Result program",
    );
}

#[test]
fn emit_json_mir_generic_option_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_enum_option.zen",
        "tests/fixtures/ir_json/mir_generic_option.golden.json",
        "generic Option program",
    );
}

#[test]
fn emit_json_mir_generic_option_multi_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_enum_multi_specialization.zen",
        "tests/fixtures/ir_json/mir_generic_option_multi.golden.json",
        "generic Option multi-specialization",
    );
}

#[test]
fn emit_json_mir_duplicate_generic_enum_variant_names_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/duplicate_enum_variant_names.zen",
        "tests/fixtures/ir_json/mir_duplicate_generic_enum_variant_names.golden.json",
        "duplicate generic enum variant names",
    );
}

#[test]
fn emit_json_mir_generic_result_multi_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_result_enum_multi_specialization.zen",
        "tests/fixtures/ir_json/mir_generic_result_multi.golden.json",
        "generic Result multi-specialization",
    );
}

#[test]
fn emit_json_mir_multi_file_generic_result_multi_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/multi_file_generic_result_enum_multi_specialization/main.zen",
        "tests/fixtures/ir_json/mir_multi_file_generic_result_multi_specialization.golden.json",
        "multi-file generic Result multi-specialization",
    );
}
