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
    fn test_multi_file_imports() => "multi_file/main.zen", "37\n";
    fn test_multi_file_type_impl_imports() => "multi_file_type_impl/main.zen", "34\n";
    fn test_multi_file_type_impl_imported_type_dependency_imports() => "multi_file_type_impl_imported_type_dependency/main.zen", "61\n";
    fn test_multi_file_type_impl_return_enum_dependency_imports() => "multi_file_type_impl_return_enum_dependency/main.zen", "101\n";
    fn test_multi_file_type_method_imports() => "multi_file_type_method/main.zen", "13\n";
    fn test_multi_file_type_method_worklist_imports() => "multi_file_type_method_worklist/main.zen", "31\n";
    fn test_multi_file_type_method_method_dependency_imports() => "multi_file_type_method_method_dependency/main.zen", "47\n";
    fn test_multi_file_type_method_imported_dependency_imports() => "multi_file_type_method_imported_dependency/main.zen", "59\n";
    fn test_multi_file_type_method_return_enum_dependency_imports() => "multi_file_type_method_return_enum_dependency/main.zen", "97\n";
    fn test_multi_file_type_method_nested_result_dependency_imports() => "multi_file_type_method_nested_result_dependency/main.zen", "109\n7\n";
    fn test_multi_file_behavior_bound_imports() => "multi_file_behavior_bound/main.zen", "11\n";
    fn test_multi_file_behavior_inheritance_imports() => "multi_file_behavior_inheritance/main.zen", "encoded\npretty\nfancy\n";
    fn test_multi_file_imported_behavior_impls() => "multi_file_imported_behavior_impl/main.zen", "encoded\n";
    fn test_multi_file_imported_behavior_defaults() => "multi_file_imported_behavior_default/main.zen", "default-json\n";
    fn test_multi_file_imported_generic_behavior_defaults() => "multi_file_imported_generic_behavior_default/main.zen", "imported-default\n";
    fn test_multi_file_imported_generic_target_behavior_association() => "multi_file_imported_generic_target_behavior_association/main.zen", "41\ntrue\n";
    fn test_multi_file_imported_generic_target_default_method() => "multi_file_imported_generic_target_default_method/main.zen", "box\nbox\n";
    fn test_multi_file_imported_impl_with_imported_behavior() => "multi_file_imported_impl_imported_behavior/main.zen", "encoded\npretty\n";
    fn test_multi_file_imported_child_parent_dispatch() => "multi_file_imported_child_parent_dispatch/main.zen", "encoded\npretty\n";
    fn test_multi_file_imported_behavior_requires() => "multi_file_imported_behavior_requires/main.zen", "required\n";
    fn test_multi_file_imported_behavior_requires_inherited() => "multi_file_imported_behavior_requires_inherited/main.zen", "inherited-required\n";
    fn test_multi_file_imported_function_imported_behavior_bound() => "multi_file_imported_function_imported_behavior_bound/main.zen", "97\n";
    fn test_multi_file_imported_function_return_type_dependency() => "multi_file_imported_function_return_type_dependency/main.zen", "101\n";
    fn test_multi_file_imported_function_param_type_dependency() => "multi_file_imported_function_param_type_dependency/main.zen", "127\n";
    fn test_multi_file_imported_function_imported_return_type_behavior() => "multi_file_imported_function_imported_return_type_behavior/main.zen", "113\n";
}
