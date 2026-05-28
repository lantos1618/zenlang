use super::assert_typed_golden;

#[test]
fn emit_json_typed_generic_enum_schemas_match_golden() {
    for (source, golden_stem, description) in [
        (
            "generic_enum_option.zen",
            "generic_option",
            "generic Option program",
        ),
        (
            "generic_enum_multi_specialization.zen",
            "generic_option_multi",
            "generic Option multi-specialization",
        ),
        (
            "duplicate_enum_variant_names.zen",
            "duplicate_generic_enum_variant_names",
            "duplicate generic enum variant names",
        ),
        (
            "generic_result_enum.zen",
            "generic_result",
            "generic Result program",
        ),
        (
            "generic_result_enum_multi_specialization.zen",
            "generic_result_multi",
            "generic Result multi-specialization",
        ),
        (
            "multi_file_generic_result_enum_multi_specialization/main.zen",
            "multi_file_generic_result_multi_specialization",
            "multi-file generic Result multi-specialization",
        ),
        (
            "multi_file_generic_result_error_multi_specialization/main.zen",
            "multi_file_generic_result_error_multi_specialization",
            "multi-file generic Result error-type multi-specialization",
        ),
        (
            "multi_file_generic_enum_method/main.zen",
            "multi_file_generic_enum_method",
            "multi-file generic enum method",
        ),
        (
            "generic_result_enum_method.zen",
            "generic_result_method",
            "generic Result method",
        ),
        (
            "multi_file_generic_result_enum_method/main.zen",
            "multi_file_generic_result_method",
            "multi-file generic Result method",
        ),
        (
            "generic_nested_result_enum.zen",
            "nested_generic_result",
            "nested generic Result program",
        ),
    ] {
        assert_typed_golden(source, golden_stem, description);
    }
}
