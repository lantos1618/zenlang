use super::assert_multi_file_fixture;

#[test]
fn test_multi_file_behavior_bound_imports() {
    assert_multi_file_fixture("multi_file_behavior_bound", "11\n");
}

#[test]
fn test_multi_file_behavior_inheritance_imports() {
    assert_multi_file_fixture(
        "multi_file_behavior_inheritance",
        "encoded\npretty\nfancy\n",
    );
}

#[test]
fn test_multi_file_imported_behavior_impls() {
    assert_multi_file_fixture("multi_file_imported_behavior_impl", "encoded\n");
}

#[test]
fn test_multi_file_imported_behavior_defaults() {
    assert_multi_file_fixture("multi_file_imported_behavior_default", "default-json\n");
}

#[test]
fn test_multi_file_imported_generic_behavior_defaults() {
    assert_multi_file_fixture(
        "multi_file_imported_generic_behavior_default",
        "imported-default\n",
    );
}

#[test]
fn test_multi_file_imported_impl_with_imported_behavior() {
    assert_multi_file_fixture(
        "multi_file_imported_impl_imported_behavior",
        "encoded\npretty\n",
    );
}

#[test]
fn test_multi_file_imported_child_parent_dispatch() {
    assert_multi_file_fixture(
        "multi_file_imported_child_parent_dispatch",
        "encoded\npretty\n",
    );
}

#[test]
fn test_multi_file_imported_behavior_requires() {
    assert_multi_file_fixture("multi_file_imported_behavior_requires", "required\n");
}

#[test]
fn test_multi_file_imported_behavior_requires_inherited() {
    assert_multi_file_fixture(
        "multi_file_imported_behavior_requires_inherited",
        "inherited-required\n",
    );
}
