use super::{assert_layout_matches_fixture, fixture};

#[test]
fn emit_json_layout_generic_schemas_match_golden() {
    for (source, description, golden_stem) in [
        (
            "tests/zen/generic_result_enum.zen",
            "generic Result program input",
            "generic_result",
        ),
        (
            "tests/zen/generic_enum_option.zen",
            "generic Option program input",
            "generic_option",
        ),
        (
            "tests/zen/generic_nested_result_enum.zen",
            "nested generic Result program input",
            "generic_enum_method_nested_result",
        ),
        (
            "tests/zen/generic_enum_method_nested_result.zen",
            "generic enum method nested Result program input",
            "generic_enum_method_nested_result",
        ),
        (
            "tests/zen/multi_file_generic_enum_method/main.zen",
            "multi-file generic enum method program input",
            "generic_option",
        ),
        (
            "tests/zen/multi_file_generic_result_enum_method/main.zen",
            "multi-file generic Result method program input",
            "generic_result",
        ),
        (
            "tests/zen/multi_file_imported_generic_function_return_enum_dependency/main.zen",
            "multi-file generic function return enum program input",
            "generic_option",
        ),
        (
            "tests/zen/multi_file_type_method_nested_result_dependency/main.zen",
            "multi-file generic method nested Result program input",
            "multi_file_generic_method_nested_result",
        ),
        (
            "tests/zen/multi_file_generic_result_enum_multi_specialization/main.zen",
            "multi-file generic Result multi-specialization program input",
            "multi_file_generic_result_multi_specialization",
        ),
        (
            "tests/zen/multi_file_generic_result_error_multi_specialization/main.zen",
            "multi-file generic Result error multi-specialization program input",
            "multi_file_generic_result_error_multi_specialization",
        ),
        (
            "tests/zen/multi_file_generic_imported_type_same_name_collision/main.zen",
            "multi-file imported generic type same-name program input",
            "multi_file_generic_imported_type_same_name",
        ),
        (
            "tests/zen/multi_file_generic_imported_scoped_type_inference/main.zen",
            "multi-file imported generic scoped type inference program input",
            "multi_file_generic_imported_scoped_type_inference",
        ),
    ] {
        assert_layout_matches_fixture(&fixture(source), description, golden_stem);
    }
}
