use super::support::*;

#[path = "generic_specializations/behavior_bounds.rs"]
mod behavior_bounds;
#[path = "generic_specializations/enum_generated_c.rs"]
mod enum_generated_c;
#[path = "generic_specializations/method_worklist_generated_c.rs"]
mod method_worklist_generated_c;
#[path = "generic_specializations/multifile_generated_c.rs"]
mod multifile_generated_c;

#[test]
fn generic_specializations_emit_each_generated_c_definition_once() {
    let fixtures = [
        "generic_enum_method.zen",
        "generic_enum_multi_specialization.zen",
        "generic_enum_option.zen",
        "generic_identity.zen",
        "generic_method.zen",
        "generic_method_nested_result.zen",
        "generic_method_self.zen",
        "generic_method_worklist.zen",
        "generic_nested_result_enum.zen",
        "generic_result_enum.zen",
        "generic_result_enum_method.zen",
        "generic_result_enum_multi_specialization.zen",
        "generic_struct.zen",
        "generic_ufc_function.zen",
        "generic_vec.zen",
        "generic_worklist.zen",
        "generic_worklist_dedup.zen",
        "multi_file_generic/main.zen",
        "multi_file_generic_enum_method/main.zen",
        "multi_file_generic_imported_transitive_dependency/main.zen",
        "multi_file_generic_imported_type_dependency/main.zen",
        "multi_file_generic_imported_worklist_chain/main.zen",
        "multi_file_generic_result_enum_method/main.zen",
        "multi_file_generic_result_enum_multi_specialization/main.zen",
        "multi_file_imported_generic_function_return_enum_dependency/main.zen",
        "multi_file_type_impl_return_enum_dependency/main.zen",
        "multi_file_type_method_nested_result_dependency/main.zen",
        "multi_file_type_method_return_enum_dependency/main.zen",
        "multi_file_type_method_worklist/main.zen",
    ];

    for fixture in fixtures {
        let c_source = compile_to_c_with_generated_call_check(&test_dir().join(fixture));
        assert_generated_c_function_definitions_are_unique(&c_source);
    }
}
