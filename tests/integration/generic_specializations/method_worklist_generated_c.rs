use super::*;

#[test]
fn method_and_worklist_specializations_do_not_emit_unspecialized_c_symbols() {
    let c_source = compile_to_c_with_generated_call_check(&test_dir().join("generic_method.zen"));
    assert!(c_source.contains("int32_t Box_get_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_get_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_get_i32");
    assert!(!c_source.contains("Box_T"));
    assert!(!c_source.contains("T Box_get"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_method_self.zen"));
    assert!(c_source.contains("Box_i32 Box_copy_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_copy_i32(box)"));
    assert!(c_source.contains("Box_Option_i32 Box_copy_Option_i32(Box_Option_i32 self)"));
    assert!(c_source.contains("Box_copy_Option_i32(nested)"));
    assert!(c_source.contains("Option_i32 Option_copy_i32(Option_i32 self)"));
    assert!(c_source.contains("Option_copy_i32(option)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_copy_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_copy_Option_i32");
    assert_c_call_resolves_to_definition(&c_source, "Option_copy_i32");
    assert!(
        c_source
            .find("struct Option_i32")
            .expect("Option_i32 struct")
            < c_source
                .find("struct Box_Option_i32")
                .expect("Box_Option_i32 struct")
    );
    assert!(!c_source.contains("Box_copy(box"));
    assert!(!c_source.contains("Unknown"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_method_worklist.zen"));
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t Box_get_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("inner_i32(self.value)"));
    assert!(c_source.contains("Box_get_inner_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_get_inner_i32");
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("inner_T"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("generic_method_method_worklist.zen"),
    );
    assert!(c_source.contains("int32_t Box_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("int32_t Box_get_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_inner_i32(self)"));
    assert!(c_source.contains("Box_get_inner_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_get_inner_i32");
    assert_c_function_definition_count(&c_source, "Box_inner_i32", 1);
    assert_c_function_definition_count(&c_source, "Box_get_inner_i32", 1);
    assert!(!c_source.contains("T Box_inner"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("generic_method_nested_result.zen"),
    );
    assert!(c_source
        .contains("typedef struct Result_Option_i32_StaticString Result_Option_i32_StaticString;"));
    assert!(c_source.contains("Result_Option_i32_StaticString Box_wrap_result_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_wrap_result_i32(box)"));
    assert!(c_source.contains("unwrap_result_Option_i32_StaticString(wrapped,"));
    assert!(c_source.contains("unwrap_option_i32(some, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_wrap_result_i32");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_result_Option_i32_StaticString");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_option_i32");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T Box_wrap_result"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("type_impl_methods.zen"));
    assert!(c_source.contains("int32_t Point_get(Point self)"));
    assert!(c_source.contains("int32_t Point_keep_i32(Point self, int32_t value)"));
    assert!(c_source.contains("Point_get(point)"));
    assert!(c_source.contains("Point_keep_i32(point, 7LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_get");
    assert_c_call_resolves_to_definition(&c_source, "Point_keep_i32");
    assert!(!c_source.contains("T Point_keep"));
    assert!(!c_source.contains("Point_keep(point"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_type_impl_methods.zen"));
    assert!(c_source.contains("int32_t Box_get_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_i32 Box_replace_i32(Box_i32 self, int32_t value)"));
    assert!(c_source.contains("Box_get_i32(next)"));
    assert!(c_source.contains("Box_replace_i32(box, 42LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_get_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_replace_i32");
    assert!(!c_source.contains("Box_T"));
    assert!(!c_source.contains("T Box_get"));
    assert!(!c_source.contains("T Box_replace"));

    let c_source = compile_to_c_with_generated_call_check(&test_dir().join("generic_vec.zen"));
    assert!(c_source.contains("int32_t Vec_len_i32(Vec_i32 self)"));
    assert!(c_source.contains("int32_t Vec_len_StaticString(Vec_StaticString self)"));
    assert!(c_source.contains("Vec_len_i32(ints)"));
    assert!(c_source.contains("Vec_len_StaticString(words)"));
    assert_c_call_resolves_to_definition(&c_source, "Vec_len_i32");
    assert_c_call_resolves_to_definition(&c_source, "Vec_len_StaticString");
    assert!(!c_source.contains("Vec_T"));
    assert!(!c_source.contains("T Vec_len"));

    let c_source = compile_to_c_with_generated_call_check(&test_dir().join("generic_worklist.zen"));
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t outer_i32(int32_t value)"));
    assert!(c_source.contains("inner_i32(value)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "outer_i32");
    assert_c_function_definition_count(&c_source, "inner_i32", 1);
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("inner_T"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_worklist_dedup.zen"));
    assert!(c_source.contains("int32_t left_i32(int32_t value)"));
    assert!(c_source.contains("int32_t right_i32(int32_t value)"));
    assert!(c_source.contains("inner_i32(value)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "left_i32");
    assert_c_call_resolves_to_definition(&c_source, "right_i32");
    assert_c_function_definition_count(&c_source, "inner_i32", 1);
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("inner_T"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_recursive_function.zen"));
    assert!(c_source.contains("int32_t repeat_i32(int32_t value, int32_t remaining)"));
    assert!(c_source.contains("repeat_i32(value, (remaining - 1LL))"));
    assert!(c_source.contains("repeat_i32(41LL, 3LL)"));
    assert_c_call_resolves_to_definition(&c_source, "repeat_i32");
    assert_c_function_definition_count(&c_source, "repeat_i32", 1);
    assert!(!c_source.contains("T repeat"));
    assert!(!c_source.contains("repeat_T"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_recursive_method.zen"));
    assert!(c_source.contains("int32_t Box_repeat_i32(Box_i32 self, int32_t remaining)"));
    assert!(c_source.contains("Box_repeat_i32(self, (remaining - 1LL))"));
    assert!(c_source.contains("Box_repeat_i32(box, 3LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_repeat_i32");
    assert_c_function_definition_count(&c_source, "Box_repeat_i32", 1);
    assert!(!c_source.contains("T Box_repeat"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_ufc_function.zen"));
    assert!(c_source.contains("int32_t id_i32(int32_t value)"));
    assert!(c_source.contains("id_i32(12LL)"));
    assert_c_call_resolves_to_definition(&c_source, "id_i32");
    assert!(!c_source.contains("id(12LL)"));
    assert!(!c_source.contains("T id"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_ufc_dedup.zen"));
    assert!(c_source.contains("int32_t id_i32(int32_t value)"));
    assert!(c_source.contains("id_i32(12LL)"));
    assert!(c_source.contains("id_i32(30LL)"));
    assert_c_call_resolves_to_definition(&c_source, "id_i32");
    assert_c_function_definition_count(&c_source, "id_i32", 1);
    assert!(!c_source.contains("id(12LL)"));
    assert!(!c_source.contains("id(30LL)"));
    assert!(!c_source.contains("T id"));
}
