use super::assert_multi_file_fixture;

#[test]
fn test_multi_file_imported_function_imported_behavior_bound() {
    assert_multi_file_fixture(
        "multi_file_imported_function_imported_behavior_bound",
        "97\n",
    );
}

#[test]
fn test_multi_file_imported_function_return_type_dependency() {
    assert_multi_file_fixture(
        "multi_file_imported_function_return_type_dependency",
        "101\n",
    );
}

#[test]
fn test_multi_file_imported_function_param_type_dependency() {
    assert_multi_file_fixture(
        "multi_file_imported_function_param_type_dependency",
        "127\n",
    );
}

#[test]
fn test_multi_file_imported_function_imported_return_type_behavior() {
    assert_multi_file_fixture(
        "multi_file_imported_function_imported_return_type_behavior",
        "113\n",
    );
}

#[test]
fn test_multi_file_imported_generic_function_return_enum_dependency() {
    assert_multi_file_fixture(
        "multi_file_imported_generic_function_return_enum_dependency",
        "107\n",
    );
}
