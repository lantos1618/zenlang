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
