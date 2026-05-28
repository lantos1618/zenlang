use super::*;

macro_rules! fixture_output_tests {
    ($(fn $name:ident() => $fixture:literal, $expected:literal;)+) => {$(
        #[test]
        fn $name() {
            assert_eq!(compile_and_run(&test_dir().join($fixture)), $expected);
        }
    )+};
}

fixture_output_tests! {
    fn test_multi_file_generic_imports() => "multi_file_generic/main.zen", "42\n7\n5\n9\n";
    fn test_multi_file_generic_imported_type_dependency_imports() => "multi_file_generic_imported_type_dependency/main.zen", "73\n";
    fn test_multi_file_generic_imported_worklist_chain_imports() => "multi_file_generic_imported_worklist_chain/main.zen", "83\n";
    fn test_multi_file_generic_imported_worklist_multi_specialization_imports() => "multi_file_generic_imported_worklist_multi_specialization/main.zen", "83\ntrue\n";
    fn test_multi_file_generic_imported_diamond_same_name_imports() => "multi_file_generic_imported_diamond_same_name/main.zen", "11\n29\n";
    fn test_multi_file_generic_imported_type_same_name_collision_imports() => "multi_file_generic_imported_type_same_name_collision/main.zen", "11\n29\n";
    fn test_multi_file_generic_recursive_function_imports() => "multi_file_generic_recursive_function/main.zen", "97\n";
    fn test_multi_file_generic_imported_transitive_dependency_imports() => "multi_file_generic_imported_transitive_dependency/main.zen", "89\n";
    fn test_multi_file_generic_enum_method_imports() => "multi_file_generic_enum_method/main.zen", "21\n89\n";
    fn test_multi_file_generic_enum_method_worklist_imports() => "multi_file_generic_enum_method_worklist/main.zen", "31\n97\ntrue\ntrue\n";
    fn test_multi_file_generic_result_enum_method_imports() => "multi_file_generic_result_enum_method/main.zen", "55\n144\n";
    fn test_multi_file_generic_result_enum_multi_specialization_imports() => "multi_file_generic_result_enum_multi_specialization/main.zen", "55\n144\nfalse\ntrue\n";
    fn test_multi_file_generic_result_error_multi_specialization_imports() => "multi_file_generic_result_error_multi_specialization/main.zen", "true\nfalse\n44\n88\n";
    fn test_multi_file_imported_scoped_generic_type_inference_ufc() => "multi_file_generic_imported_scoped_type_inference/main.zen", "1\n60\n";
    fn test_multi_file_imported_generic_function_return_enum_dependency() => "multi_file_imported_generic_function_return_enum_dependency/main.zen", "107\n";
}
