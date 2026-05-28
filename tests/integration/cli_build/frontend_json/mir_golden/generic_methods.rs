use super::assert_mir_golden;

#[test]
fn emit_json_mir_generic_method_schemas_match_golden() {
    for (source, golden_stem, description) in [
        (
            "tests/zen/generic_method.zen",
            "generic_method",
            "generic method input",
        ),
        (
            "tests/zen/generic_type_impl_methods.zen",
            "generic_type_impl_methods",
            "generic type impl methods input",
        ),
        (
            "tests/zen/generic_method_self.zen",
            "generic_self_method",
            "generic Self method input",
        ),
        (
            "tests/zen/generic_method_worklist.zen",
            "generic_method_worklist",
            "generic method worklist input",
        ),
        (
            "tests/zen/generic_method_method_worklist.zen",
            "generic_method_method_worklist",
            "generic method-to-method worklist input",
        ),
        (
            "tests/zen/multi_file_type_method_worklist/main.zen",
            "multi_file_type_method_worklist",
            "multi-file type method worklist input",
        ),
        (
            "tests/zen/multi_file_generic_imported_type_dependency/main.zen",
            "multi_file_generic_imported_type_dependency",
            "multi-file generic imported type dependency input",
        ),
        (
            "tests/zen/multi_file_type_impl_imported_type_dependency/main.zen",
            "multi_file_type_impl_imported_type_dependency",
            "multi-file type impl imported type dependency input",
        ),
        (
            "tests/zen/multi_file_type_method_return_enum_dependency/main.zen",
            "multi_file_type_method_return_enum",
            "multi-file type method return enum input",
        ),
        (
            "tests/zen/generic_recursive_method.zen",
            "generic_recursive_method",
            "generic recursive method input",
        ),
        (
            "tests/zen/generic_method_nested_result.zen",
            "generic_method_nested_result",
            "generic method nested result input",
        ),
        (
            "tests/zen/generic_enum_method_nested_result.zen",
            "generic_enum_method_nested_result",
            "generic enum method nested result input",
        ),
        (
            "tests/zen/multi_file_type_method_nested_result_dependency/main.zen",
            "multi_file_generic_method_nested_result",
            "multi-file generic method nested result input",
        ),
        (
            "tests/zen/multi_file_generic_result_enum_method/main.zen",
            "multi_file_generic_result_method",
            "multi-file generic Result method input",
        ),
        (
            "tests/zen/generic_result_enum_method.zen",
            "generic_result_method",
            "generic Result method program input",
        ),
        (
            "tests/zen/generic_nested_result_enum.zen",
            "nested_generic_result",
            "nested generic Result program input",
        ),
    ] {
        assert_mir_golden(source, golden_stem, description);
    }
}
