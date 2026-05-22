use super::assert_multi_file_fixture;

#[test]
fn test_multi_file_generic_imports() {
    assert_multi_file_fixture("multi_file_generic", "42\n7\n5\n9\n");
}

#[test]
fn test_multi_file_generic_imported_type_dependency_imports() {
    assert_multi_file_fixture("multi_file_generic_imported_type_dependency", "73\n");
}

#[test]
fn test_multi_file_generic_imported_worklist_chain_imports() {
    assert_multi_file_fixture("multi_file_generic_imported_worklist_chain", "83\n");
}

#[test]
fn test_multi_file_generic_imported_transitive_dependency_imports() {
    assert_multi_file_fixture("multi_file_generic_imported_transitive_dependency", "89\n");
}

#[test]
fn test_multi_file_generic_enum_method_imports() {
    assert_multi_file_fixture("multi_file_generic_enum_method", "21\n89\n");
}

#[test]
fn test_multi_file_generic_result_enum_method_imports() {
    assert_multi_file_fixture("multi_file_generic_result_enum_method", "55\n144\n");
}

#[test]
fn test_multi_file_generic_result_enum_multi_specialization_imports() {
    assert_multi_file_fixture(
        "multi_file_generic_result_enum_multi_specialization",
        "55\n144\nfalse\ntrue\n",
    );
}
