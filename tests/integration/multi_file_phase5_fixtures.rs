use super::*;

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
fn test_multi_file_generic_imported_type_same_name_collision_imports() {
    let zen_path = test_dir().join("multi_file_generic_imported_type_same_name_collision/main.zen");
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
fn test_multi_file_imported_scoped_generic_type_inference_ufc() {
    let zen_path = test_dir().join("multi_file_generic_imported_scoped_type_inference/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "1\n60\n");
}

#[test]
fn test_multi_file_imported_generic_function_return_enum_dependency() {
    let zen_path =
        test_dir().join("multi_file_imported_generic_function_return_enum_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "107\n");
}
