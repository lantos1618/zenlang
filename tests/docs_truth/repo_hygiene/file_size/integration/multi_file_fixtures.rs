use super::*;

#[test]
fn multi_file_fixture_tests_stay_split_by_fixture_family() {
    let root = read("tests/integration/multi_file_fixtures.rs");
    let basic = read("tests/integration/multi_file_fixtures/basic.rs");
    let behavior_imports = read("tests/integration/multi_file_fixtures/behavior_imports.rs");
    let function_dependencies =
        read("tests/integration/multi_file_fixtures/function_dependencies.rs");
    let generic_imports = read("tests/integration/multi_file_fixtures/generic_imports.rs");
    let type_impls = read("tests/integration/multi_file_fixtures/type_impls.rs");
    let type_methods = read("tests/integration/multi_file_fixtures/type_methods.rs");

    assert!(
        root.lines().count() < 60,
        "multi_file_fixtures.rs should route focused multi-file fixture modules"
    );
    assert!(
        !root.contains("#[test]"),
        "multi_file_fixtures.rs should not own concrete fixture tests"
    );
    for module in [
        r#"#[path = "multi_file_fixtures/basic.rs"]"#,
        r#"#[path = "multi_file_fixtures/behavior_imports.rs"]"#,
        r#"#[path = "multi_file_fixtures/function_dependencies.rs"]"#,
        r#"#[path = "multi_file_fixtures/generic_imports.rs"]"#,
        r#"#[path = "multi_file_fixtures/type_impls.rs"]"#,
        r#"#[path = "multi_file_fixtures/type_methods.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "multi_file_fixtures.rs should include focused module path `{module}`"
        );
    }

    assert_multi_file_tests_live_in(&root, &basic, &["test_multi_file_imports"], "basic.rs");
    assert_multi_file_tests_live_in(
        &root,
        &generic_imports,
        &[
            "test_multi_file_generic_imports",
            "test_multi_file_generic_imported_type_dependency_imports",
            "test_multi_file_generic_imported_worklist_chain_imports",
            "test_multi_file_generic_imported_transitive_dependency_imports",
            "test_multi_file_generic_enum_method_imports",
            "test_multi_file_generic_result_enum_method_imports",
            "test_multi_file_generic_result_enum_multi_specialization_imports",
        ],
        "generic_imports.rs",
    );
    assert_multi_file_tests_live_in(
        &root,
        &type_impls,
        &[
            "test_multi_file_type_impl_imports",
            "test_multi_file_type_impl_imported_type_dependency_imports",
            "test_multi_file_type_impl_return_enum_dependency_imports",
        ],
        "type_impls.rs",
    );
    assert_multi_file_tests_live_in(
        &root,
        &type_methods,
        &[
            "test_multi_file_type_method_imports",
            "test_multi_file_type_method_worklist_imports",
            "test_multi_file_type_method_method_dependency_imports",
            "test_multi_file_type_method_imported_dependency_imports",
            "test_multi_file_type_method_return_enum_dependency_imports",
            "test_multi_file_type_method_nested_result_dependency_imports",
        ],
        "type_methods.rs",
    );
    assert_multi_file_tests_live_in(
        &root,
        &behavior_imports,
        &[
            "test_multi_file_behavior_bound_imports",
            "test_multi_file_behavior_inheritance_imports",
            "test_multi_file_imported_behavior_impls",
            "test_multi_file_imported_behavior_defaults",
            "test_multi_file_imported_generic_behavior_defaults",
            "test_multi_file_imported_impl_with_imported_behavior",
            "test_multi_file_imported_child_parent_dispatch",
            "test_multi_file_imported_behavior_requires",
            "test_multi_file_imported_behavior_requires_inherited",
        ],
        "behavior_imports.rs",
    );
    assert_multi_file_tests_live_in(
        &root,
        &function_dependencies,
        &[
            "test_multi_file_imported_function_imported_behavior_bound",
            "test_multi_file_imported_function_return_type_dependency",
            "test_multi_file_imported_function_param_type_dependency",
            "test_multi_file_imported_function_imported_return_type_behavior",
            "test_multi_file_imported_generic_function_return_enum_dependency",
        ],
        "function_dependencies.rs",
    );
}

fn assert_multi_file_tests_live_in(
    root: &str,
    focused_module: &str,
    test_names: &[&str],
    focused_path: &str,
) {
    for test_name in test_names {
        let fn_name = format!("fn {test_name}");
        assert!(
            !root.contains(&fn_name),
            "multi-file fixture test should move out of the root module: {test_name}"
        );
        assert!(
            focused_module.contains(&fn_name),
            "{focused_path} should keep multi-file fixture test: {test_name}"
        );
    }
}
