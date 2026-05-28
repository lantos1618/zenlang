use super::assert_typed_golden;

#[test]
fn emit_json_typed_method_schemas_match_golden() {
    for (source, golden_stem, description) in [
        ("generic_method.zen", "generic_method", "generic method"),
        (
            "generic_type_impl_methods.zen",
            "generic_type_impl_methods",
            "generic type impl methods",
        ),
        (
            "generic_method_self.zen",
            "generic_self_method",
            "generic Self method",
        ),
        (
            "generic_method_worklist.zen",
            "generic_method_worklist",
            "generic method worklist",
        ),
        (
            "generic_method_method_worklist.zen",
            "generic_method_method_worklist",
            "generic method-to-method worklist",
        ),
        (
            "generic_recursive_method.zen",
            "generic_recursive_method",
            "generic recursive method",
        ),
        (
            "generic_method_nested_result.zen",
            "generic_method_nested_result",
            "generic method nested Result",
        ),
        (
            "generic_enum_method_nested_result.zen",
            "generic_enum_method_nested_result",
            "generic enum method nested Result",
        ),
        (
            "multi_file_type_method_nested_result_dependency/main.zen",
            "multi_file_generic_method_nested_result",
            "multi-file generic method nested Result",
        ),
    ] {
        assert_typed_golden(source, golden_stem, description);
    }
}
