use super::assert_multi_file_fixture;

#[test]
fn test_multi_file_type_impl_imports() {
    assert_multi_file_fixture("multi_file_type_impl", "34\n");
}

#[test]
fn test_multi_file_type_impl_imported_type_dependency_imports() {
    assert_multi_file_fixture("multi_file_type_impl_imported_type_dependency", "61\n");
}

#[test]
fn test_multi_file_type_impl_return_enum_dependency_imports() {
    assert_multi_file_fixture("multi_file_type_impl_return_enum_dependency", "101\n");
}
