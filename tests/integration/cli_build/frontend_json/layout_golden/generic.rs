use super::assert_layout_matches_fixture;
use super::fixture;

#[test]
fn emit_json_layout_generic_result_schema_matches_golden() {
    assert_layout_matches_fixture(
        &fixture("tests/zen/generic_result_enum.zen"),
        "generic Result program input",
        "tests/fixtures/ir_json/layout_generic_result.golden.json",
    );
}

#[test]
fn emit_json_layout_generic_option_schema_matches_golden() {
    assert_layout_matches_fixture(
        &fixture("tests/zen/generic_enum_option.zen"),
        "generic Option program input",
        "tests/fixtures/ir_json/layout_generic_option.golden.json",
    );
}

#[test]
fn emit_json_layout_nested_generic_result_schema_matches_golden() {
    assert_layout_matches_fixture(
        &fixture("tests/zen/generic_nested_result_enum.zen"),
        "nested generic Result program input",
        "tests/fixtures/ir_json/layout_nested_generic_result.golden.json",
    );
}

#[test]
fn emit_json_layout_generic_enum_method_nested_result_schema_matches_golden() {
    assert_layout_matches_fixture(
        &fixture("tests/zen/generic_enum_method_nested_result.zen"),
        "generic enum method nested Result program input",
        "tests/fixtures/ir_json/layout_generic_enum_method_nested_result.golden.json",
    );
}

#[test]
fn emit_json_layout_multi_file_generic_enum_method_schema_matches_golden() {
    assert_layout_matches_fixture(
        &fixture("tests/zen/multi_file_generic_enum_method/main.zen"),
        "multi-file generic enum method program input",
        "tests/fixtures/ir_json/layout_multi_file_generic_enum_method.golden.json",
    );
}

#[test]
fn emit_json_layout_multi_file_generic_result_method_schema_matches_golden() {
    assert_layout_matches_fixture(
        &fixture("tests/zen/multi_file_generic_result_enum_method/main.zen"),
        "multi-file generic Result method program input",
        "tests/fixtures/ir_json/layout_multi_file_generic_result_method.golden.json",
    );
}

#[test]
fn emit_json_layout_multi_file_generic_function_return_enum_schema_matches_golden() {
    assert_layout_matches_fixture(
        &fixture("tests/zen/multi_file_imported_generic_function_return_enum_dependency/main.zen"),
        "multi-file generic function return enum program input",
        "tests/fixtures/ir_json/layout_multi_file_generic_function_return_enum.golden.json",
    );
}

#[test]
fn emit_json_layout_multi_file_generic_method_nested_result_schema_matches_golden() {
    assert_layout_matches_fixture(
        &fixture("tests/zen/multi_file_type_method_nested_result_dependency/main.zen"),
        "multi-file generic method nested Result program input",
        "tests/fixtures/ir_json/layout_multi_file_generic_method_nested_result.golden.json",
    );
}

#[test]
fn emit_json_layout_multi_file_generic_result_multi_specialization_schema_matches_golden() {
    assert_layout_matches_fixture(
        &fixture("tests/zen/multi_file_generic_result_enum_multi_specialization/main.zen"),
        "multi-file generic Result multi-specialization program input",
        "tests/fixtures/ir_json/layout_multi_file_generic_result_multi_specialization.golden.json",
    );
}
