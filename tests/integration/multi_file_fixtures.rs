use super::*;

#[test]
fn test_multi_file_imports() {
    let zen_path = test_dir().join("multi_file/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "37\n");
}

#[test]
fn test_multi_file_generic_imports() {
    let zen_path = test_dir().join("multi_file_generic/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "42\n7\n5\n9\n");
}

#[test]
fn test_multi_file_generic_imported_type_dependency_imports() {
    let zen_path = test_dir().join("multi_file_generic_imported_type_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "73\n");
}

#[test]
fn test_multi_file_generic_imported_worklist_chain_imports() {
    let zen_path = test_dir().join("multi_file_generic_imported_worklist_chain/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "83\n");
}

#[test]
fn test_multi_file_generic_imported_diamond_same_name_imports() {
    let zen_path = test_dir().join("multi_file_generic_imported_diamond_same_name/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "11\n29\n");
}

#[test]
fn test_multi_file_generic_recursive_function_imports() {
    let zen_path = test_dir().join("multi_file_generic_recursive_function/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "97\n");
}

#[test]
fn test_multi_file_generic_imported_transitive_dependency_imports() {
    let zen_path = test_dir().join("multi_file_generic_imported_transitive_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "89\n");
}

#[test]
fn test_multi_file_generic_enum_method_imports() {
    let zen_path = test_dir().join("multi_file_generic_enum_method/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "21\n89\n");
}

#[test]
fn test_multi_file_generic_result_enum_method_imports() {
    let zen_path = test_dir().join("multi_file_generic_result_enum_method/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "55\n144\n");
}

#[test]
fn test_multi_file_generic_result_enum_multi_specialization_imports() {
    let zen_path = test_dir().join("multi_file_generic_result_enum_multi_specialization/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "55\n144\nfalse\ntrue\n");
}

#[test]
fn test_multi_file_type_impl_imports() {
    let zen_path = test_dir().join("multi_file_type_impl/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "34\n");
}

#[test]
fn test_multi_file_type_impl_imported_type_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_impl_imported_type_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "61\n");
}

#[test]
fn test_multi_file_type_impl_return_enum_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_impl_return_enum_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "101\n");
}

#[test]
fn test_multi_file_type_method_imports() {
    let zen_path = test_dir().join("multi_file_type_method/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "13\n");
}

#[test]
fn test_multi_file_type_method_worklist_imports() {
    let zen_path = test_dir().join("multi_file_type_method_worklist/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "31\n");
}

#[test]
fn test_multi_file_type_method_method_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_method_method_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "47\n");
}

#[test]
fn test_multi_file_type_method_imported_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_method_imported_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "59\n");
}

#[test]
fn test_multi_file_type_method_return_enum_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_method_return_enum_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "97\n");
}

#[test]
fn test_multi_file_type_method_nested_result_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_method_nested_result_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "109\n7\n");
}

#[test]
fn test_multi_file_behavior_bound_imports() {
    let zen_path = test_dir().join("multi_file_behavior_bound/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "11\n");
}

#[test]
fn test_multi_file_behavior_inheritance_imports() {
    let zen_path = test_dir().join("multi_file_behavior_inheritance/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "encoded\npretty\nfancy\n");
}

#[test]
fn test_multi_file_imported_behavior_impls() {
    let zen_path = test_dir().join("multi_file_imported_behavior_impl/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "encoded\n");
}

#[test]
fn test_multi_file_imported_behavior_defaults() {
    let zen_path = test_dir().join("multi_file_imported_behavior_default/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "default-json\n");
}

#[test]
fn test_multi_file_imported_generic_behavior_defaults() {
    let zen_path = test_dir().join("multi_file_imported_generic_behavior_default/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "imported-default\n");
}

#[test]
fn test_multi_file_imported_impl_with_imported_behavior() {
    let zen_path = test_dir().join("multi_file_imported_impl_imported_behavior/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "encoded\npretty\n");
}

#[test]
fn test_multi_file_imported_child_parent_dispatch() {
    let zen_path = test_dir().join("multi_file_imported_child_parent_dispatch/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "encoded\npretty\n");
}

#[test]
fn test_multi_file_imported_behavior_requires() {
    let zen_path = test_dir().join("multi_file_imported_behavior_requires/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "required\n");
}

#[test]
fn test_multi_file_imported_behavior_requires_inherited() {
    let zen_path = test_dir().join("multi_file_imported_behavior_requires_inherited/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "inherited-required\n");
}

#[test]
fn test_multi_file_imported_function_imported_behavior_bound() {
    let zen_path = test_dir().join("multi_file_imported_function_imported_behavior_bound/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "97\n");
}

#[test]
fn test_multi_file_imported_function_return_type_dependency() {
    let zen_path = test_dir().join("multi_file_imported_function_return_type_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "101\n");
}

#[test]
fn test_multi_file_imported_function_param_type_dependency() {
    let zen_path = test_dir().join("multi_file_imported_function_param_type_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "127\n");
}

#[test]
fn test_multi_file_imported_function_imported_return_type_behavior() {
    let zen_path =
        test_dir().join("multi_file_imported_function_imported_return_type_behavior/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "113\n");
}

#[test]
fn test_multi_file_imported_generic_function_return_enum_dependency() {
    let zen_path =
        test_dir().join("multi_file_imported_generic_function_return_enum_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "107\n");
}

// ── Discovery test: all .zen files have matching .expected ──────────
