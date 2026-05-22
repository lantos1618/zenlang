use super::*;

#[test]
fn multi_file_generic_method_and_worklist_specializations_do_not_emit_unspecialized_c_symbols() {
    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_generic_imported_type_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Holder_i32 Holder_i32;"));
    assert!(c_source.contains("int32_t Holder_get_i32(Holder_i32 self)"));
    assert!(c_source.contains("int32_t get_held_i32(int32_t value)"));
    assert!(c_source.contains("const Holder_i32 holder = (Holder_i32){ .value = value }"));
    assert!(c_source.contains("Holder_get_i32(holder)"));
    assert!(c_source.contains("get_held_i32(73LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Holder_get_i32");
    assert_c_call_resolves_to_definition(&c_source, "get_held_i32");
    assert!(!c_source.contains("Holder_T"));
    assert!(!c_source.contains("T Holder_get"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_generic_imported_worklist_chain/main.zen"),
    );
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t middle_i32(int32_t value)"));
    assert!(c_source.contains("int32_t outer_i32(int32_t value)"));
    assert!(c_source.contains("inner_i32(value)"));
    assert!(c_source.contains("middle_i32(value)"));
    assert!(c_source.contains("outer_i32(83LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "middle_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "outer_i32");
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("T middle"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_generic_recursive_function/main.zen"),
    );
    assert!(c_source.contains("int32_t repeat_i32(int32_t value, int32_t remaining)"));
    assert!(c_source.contains("repeat_i32(value, (remaining - 1LL))"));
    assert!(c_source.contains("repeat_i32(97LL, 4LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "repeat_i32");
    assert!(!c_source.contains("T repeat"));
    assert!(!c_source.contains("repeat_T"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_generic_imported_transitive_dependency/main.zen"),
    );
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t middle_i32(int32_t value)"));
    assert!(c_source.contains("int32_t outer_i32(int32_t value)"));
    assert!(c_source.contains("inner_i32(value)"));
    assert!(c_source.contains("middle_i32(value)"));
    assert!(c_source.contains("outer_i32(89LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "middle_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "outer_i32");
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("T middle"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("multi_file_type_impl/main.zen"));
    assert!(c_source.contains("int32_t Box_get_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_get_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_get_i32");
    assert!(!c_source.contains("Box_T"));
    assert!(!c_source.contains("T Box_get"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_impl_imported_type_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Holder_i32 Holder_i32;"));
    assert!(c_source.contains("int32_t Holder_get_i32(Holder_i32 self)"));
    assert!(c_source.contains("int32_t Box_get_held_i32(Box_i32 self)"));
    assert!(c_source.contains("const Holder_i32 holder = (Holder_i32){ .value = self.value }"));
    assert!(c_source.contains("Holder_get_i32(holder)"));
    assert!(c_source.contains("Box_get_held_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Holder_get_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_get_held_i32");
    assert!(!c_source.contains("Holder_T"));
    assert!(!c_source.contains("T Holder_get"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_impl_return_enum_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("Option_i32 Box_wrap_i32(Box_i32 self)"));
    assert!(c_source.contains("int32_t Box_value_or_i32(Box_i32 self, int32_t fallback)"));
    assert!(c_source.contains("Box_wrap_i32(self)"));
    assert!(c_source.contains("Box_value_or_i32(box, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_wrap_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_value_or_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T Box_wrap"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("multi_file_type_method/main.zen"));
    assert!(c_source.contains("int32_t Point_keep_i32(Point self, int32_t value)"));
    assert!(c_source.contains("Point_keep_i32(point, 13LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_keep_i32");
    assert!(!c_source.contains("T Point_keep"));
    assert!(!c_source.contains("Point_keep(point"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_method_worklist/main.zen"),
    );
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t Box_get_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("inner_i32(self.value)"));
    assert!(c_source.contains("Box_get_inner_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_get_inner_i32");
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("inner_T"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_method_method_dependency/main.zen"),
    );
    assert!(c_source.contains("int32_t Box_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("int32_t Box_get_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_inner_i32(self)"));
    assert!(c_source.contains("Box_get_inner_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_get_inner_i32");
    assert!(!c_source.contains("T Box_inner"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_method_imported_dependency/main.zen"),
    );
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t Box_get_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("inner_i32(self.value)"));
    assert!(c_source.contains("Box_get_inner_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_get_inner_i32");
    assert!(!c_source.contains("T inner"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_method_return_enum_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("Option_i32 Box_wrap_i32(Box_i32 self)"));
    assert!(c_source.contains("int32_t Box_value_or_i32(Box_i32 self, int32_t fallback)"));
    assert!(c_source.contains("Box_wrap_i32(self)"));
    assert!(c_source.contains("Box_value_or_i32(box, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_wrap_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_value_or_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T Box_wrap"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_method_nested_result_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source
        .contains("typedef struct Result_Option_i32_StaticString Result_Option_i32_StaticString;"));
    assert!(c_source.contains("Result_Option_i32_StaticString Box_wrap_result_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_wrap_result_i32(box)"));
    assert!(c_source.contains("unwrap_result_Option_i32_StaticString(wrapped,"));
    assert!(c_source.contains("unwrap_option_i32(some, 0LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Box_wrap_result_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "unwrap_result_Option_i32_StaticString");
    assert_c_call_resolves_to_single_definition(&c_source, "unwrap_option_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("Result_Option_T"));
    assert!(!c_source.contains("T Box_wrap_result"));
}
