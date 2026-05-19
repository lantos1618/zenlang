use super::assert_typed_golden;

#[test]
fn emit_json_typed_generic_option_schema_matches_golden() {
    assert_typed_golden(
        "generic_enum_option.zen",
        "tests/fixtures/ir_json/typed_generic_option.golden.json",
        "generic Option program",
    );
}

#[test]
fn emit_json_typed_generic_option_multi_schema_matches_golden() {
    assert_typed_golden(
        "generic_enum_multi_specialization.zen",
        "tests/fixtures/ir_json/typed_generic_option_multi.golden.json",
        "generic Option multi-specialization",
    );
}

#[test]
fn emit_json_typed_generic_result_schema_matches_golden() {
    assert_typed_golden(
        "generic_result_enum.zen",
        "tests/fixtures/ir_json/typed_generic_result.golden.json",
        "generic Result program",
    );
}

#[test]
fn emit_json_typed_generic_result_multi_schema_matches_golden() {
    assert_typed_golden(
        "generic_result_enum_multi_specialization.zen",
        "tests/fixtures/ir_json/typed_generic_result_multi.golden.json",
        "generic Result multi-specialization",
    );
}

#[test]
fn emit_json_typed_generic_result_method_schema_matches_golden() {
    assert_typed_golden(
        "generic_result_enum_method.zen",
        "tests/fixtures/ir_json/typed_generic_result_method.golden.json",
        "generic Result method",
    );
}

#[test]
fn emit_json_typed_nested_generic_result_schema_matches_golden() {
    assert_typed_golden(
        "generic_nested_result_enum.zen",
        "tests/fixtures/ir_json/typed_nested_generic_result.golden.json",
        "nested generic Result program",
    );
}
