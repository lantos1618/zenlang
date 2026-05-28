use super::assert_mir_golden;

#[test]
fn emit_json_mir_generic_enum_schemas_match_golden() {
    for (source, golden_stem, description) in [
        (
            "tests/zen/generic_result_enum.zen",
            "generic_result",
            "generic Result program",
        ),
        (
            "tests/zen/generic_enum_option.zen",
            "generic_option",
            "generic Option program",
        ),
        (
            "tests/zen/generic_enum_multi_specialization.zen",
            "generic_option_multi",
            "generic Option multi-specialization",
        ),
        (
            "tests/zen/duplicate_enum_variant_names.zen",
            "duplicate_generic_enum_variant_names",
            "duplicate generic enum variant names",
        ),
        (
            "tests/zen/generic_result_enum_multi_specialization.zen",
            "generic_result_multi",
            "generic Result multi-specialization",
        ),
        (
            "tests/zen/multi_file_generic_result_enum_multi_specialization/main.zen",
            "multi_file_generic_result_multi_specialization",
            "multi-file generic Result multi-specialization",
        ),
        (
            "tests/zen/multi_file_generic_result_error_multi_specialization/main.zen",
            "multi_file_generic_result_error_multi_specialization",
            "multi-file generic Result error-type multi-specialization",
        ),
        (
            "tests/zen/multi_file_generic_enum_method/main.zen",
            "multi_file_generic_enum_method",
            "multi-file generic enum method input",
        ),
    ] {
        assert_mir_golden(source, golden_stem, description);
    }
}
