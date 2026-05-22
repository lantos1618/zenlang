use super::assert_multi_file_fixture;

#[test]
fn test_multi_file_type_method_imports() {
    assert_multi_file_fixture("multi_file_type_method", "13\n");
}

#[test]
fn test_multi_file_type_method_worklist_imports() {
    assert_multi_file_fixture("multi_file_type_method_worklist", "31\n");
}

#[test]
fn test_multi_file_type_method_method_dependency_imports() {
    assert_multi_file_fixture("multi_file_type_method_method_dependency", "47\n");
}

#[test]
fn test_multi_file_type_method_imported_dependency_imports() {
    assert_multi_file_fixture("multi_file_type_method_imported_dependency", "59\n");
}

#[test]
fn test_multi_file_type_method_return_enum_dependency_imports() {
    assert_multi_file_fixture("multi_file_type_method_return_enum_dependency", "97\n");
}

#[test]
fn test_multi_file_type_method_nested_result_dependency_imports() {
    assert_multi_file_fixture(
        "multi_file_type_method_nested_result_dependency",
        "109\n7\n",
    );
}
