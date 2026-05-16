use super::support::*;

#[test]
fn generic_specializations_emit_each_generated_c_definition_once() {
    let fixtures = [
        "generic_enum_method.zen",
        "generic_enum_option.zen",
        "generic_identity.zen",
        "generic_method.zen",
        "generic_method_nested_result.zen",
        "generic_method_self.zen",
        "generic_method_worklist.zen",
        "generic_nested_result_enum.zen",
        "generic_result_enum.zen",
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

#[test]
fn generic_specializations_do_not_emit_unspecialized_c_symbols() {
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
        &test_dir().join("generic_method_nested_result.zen"),
    );
    assert!(c_source.contains("typedef struct Result_Option_i32_str Result_Option_i32_str;"));
    assert!(c_source.contains("Result_Option_i32_str Box_wrap_result_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_wrap_result_i32(box)"));
    assert!(c_source.contains("unwrap_result_Option_i32_str(wrapped,"));
    assert!(c_source.contains("unwrap_option_i32(some, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_wrap_result_i32");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_result_Option_i32_str");
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

    let c_source = compile_to_c_with_generated_call_check(&test_dir().join("generic_vec.zen"));
    assert!(c_source.contains("int32_t Vec_len_i32(Vec_i32 self)"));
    assert!(c_source.contains("int32_t Vec_len_str(Vec_str self)"));
    assert!(c_source.contains("Vec_len_i32(ints)"));
    assert!(c_source.contains("Vec_len_str(words)"));
    assert_c_call_resolves_to_definition(&c_source, "Vec_len_i32");
    assert_c_call_resolves_to_definition(&c_source, "Vec_len_str");
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
        compile_to_c_with_generated_call_check(&test_dir().join("generic_enum_option.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("int32_t unwrap_or_i32(Option_i32 value, int32_t fallback)"));
    assert!(c_source.contains("Option_i32_Some"));
    assert!(c_source.contains("unwrap_or_i32(x, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "unwrap_or_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T unwrap_or"));
    assert!(!c_source.contains("unwrap_or(x"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_enum_method.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("int32_t Option_unwrap_or_i32(Option_i32 self, int32_t fallback)"));
    assert!(c_source.contains("Option_unwrap_or_i32(some, 0LL)"));
    assert!(c_source.contains("Option_unwrap_or_i32(none, 55LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Option_unwrap_or_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T Option_unwrap_or"));
    assert!(!c_source.contains("Option_unwrap_or(some"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("duplicate_enum_variant_names.zen"),
    );
    assert!(c_source.contains("First_i32_Some"));
    assert!(c_source.contains("First_i32_None"));
    assert!(c_source.contains("Second_bool_Some"));
    assert!(c_source.contains("Second_bool_None"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_result_enum.zen"));
    assert!(c_source.contains("typedef struct Result_i32_str Result_i32_str;"));
    assert!(c_source.contains("int32_t unwrap_or_i32_str(Result_i32_str value, int32_t fallback)"));
    assert!(c_source.contains("Result_i32_str_Err"));
    assert!(c_source.contains("unwrap_or_i32_str(err, 9LL)"));
    assert_c_call_resolves_to_definition(&c_source, "unwrap_or_i32_str");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T unwrap_or"));
    assert!(!c_source.contains("unwrap_or(err"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_nested_result_enum.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("typedef struct Result_Option_i32_str Result_Option_i32_str;"));
    assert!(c_source.contains(
        "Option_i32 unwrap_result_Option_i32_str(Result_Option_i32_str value, Option_i32 fallback)"
    ));
    assert!(c_source.contains("unwrap_result_Option_i32_str(ok,"));
    assert!(c_source.contains("unwrap_option_i32(some, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "unwrap_result_Option_i32_str");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_option_i32");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T unwrap_result"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("multi_file_generic/main.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("typedef struct Result_i32_str Result_i32_str;"));
    assert!(c_source.contains("int32_t unwrap_option_i32(Option_i32 value, int32_t fallback)"));
    assert!(
        c_source.contains("int32_t unwrap_result_i32_str(Result_i32_str value, int32_t fallback)")
    );
    assert!(c_source.contains("unwrap_option_i32(some, 0LL)"));
    assert!(c_source.contains("unwrap_result_i32_str(err, 9LL)"));
    assert_c_call_resolves_to_definition(&c_source, "unwrap_option_i32");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_result_i32_str");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T unwrap_option"));
    assert!(!c_source.contains("T unwrap_result"));
    assert!(!c_source.contains("unwrap_option(some"));
    assert!(!c_source.contains("unwrap_result(err"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_generic_enum_method/main.zen"),
    );
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("int32_t Option_unwrap_or_i32(Option_i32 self, int32_t fallback)"));
    assert!(c_source.contains("Option_unwrap_or_i32(some, 0LL)"));
    assert!(c_source.contains("Option_unwrap_or_i32(none, 89LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Option_unwrap_or_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T Option_unwrap_or"));
    assert!(!c_source.contains("Option_unwrap_or(some"));

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
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "middle_i32");
    assert_c_call_resolves_to_definition(&c_source, "outer_i32");
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("T middle"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_generic_imported_transitive_dependency/main.zen"),
    );
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t middle_i32(int32_t value)"));
    assert!(c_source.contains("int32_t outer_i32(int32_t value)"));
    assert!(c_source.contains("inner_i32(value)"));
    assert!(c_source.contains("middle_i32(value)"));
    assert!(c_source.contains("outer_i32(89LL)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "middle_i32");
    assert_c_call_resolves_to_definition(&c_source, "outer_i32");
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
    assert!(c_source.contains("typedef struct Result_Option_i32_str Result_Option_i32_str;"));
    assert!(c_source.contains("Result_Option_i32_str Box_wrap_result_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_wrap_result_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_wrap_result_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("Result_Option_T"));
    assert!(!c_source.contains("T Box_wrap_result"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_ufc_function.zen"));
    assert!(c_source.contains("int32_t id_i32(int32_t value)"));
    assert!(c_source.contains("id_i32(12LL)"));
    assert_c_call_resolves_to_definition(&c_source, "id_i32");
    assert!(!c_source.contains("id(12LL)"));
    assert!(!c_source.contains("T id"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("behavior_json_generic_bound_ufcs.zen"),
    );
    assert!(c_source.contains("Point Point_encode(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_behavior_bound/main.zen"),
    );
    assert!(c_source.contains("Point Point_encode(Point value)"));
    assert!(c_source.contains("Point encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_behavior_inheritance/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_behavior_impl/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_behavior_default/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_to_json(Point "));
    assert!(c_source.contains("zen_str render_Point(Point value)"));
    assert!(c_source.contains("Point_to_json(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_to_json");
    assert_c_call_resolves_to_definition(&c_source, "render_Point");
    assert!(!c_source.contains("T_to_json"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_generic_behavior_default/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point __arg0)"));
    assert!(c_source.contains("zen_str render_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "render_Point");
    assert!(!c_source.contains("Json_T"));
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("behavior_generic_default_method.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point __arg0)"));
    assert!(c_source.contains("Point_encode(point)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert!(!c_source.contains("Json_T"));
    assert!(!c_source.contains("T Point_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_impl_imported_behavior/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_child_parent_dispatch/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str render_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "render_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_behavior_requires/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_imported_behavior_bound/main.zen"),
    );
    assert!(c_source.contains("int32_t Point_encode(Point value)"));
    assert!(c_source.contains("int32_t encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_return_type_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Point"));
    assert!(c_source.contains("Point make_point(void)"));
    assert!(c_source.contains("int32_t Point_encode(Point value)"));
    assert!(c_source.contains("int32_t encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "make_point");
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_param_type_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Point"));
    assert!(c_source.contains("Point make_point(void)"));
    assert!(c_source.contains("int32_t Point_encode(Point value)"));
    assert!(c_source.contains("int32_t encode_point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert!(c_source.contains("encode_point(point)"));
    assert_c_call_resolves_to_definition(&c_source, "make_point");
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_imported_return_type_behavior/main.zen"),
    );
    assert!(c_source.contains("typedef struct Point"));
    assert!(c_source.contains("Point make_point(void)"));
    assert!(c_source.contains("int32_t Point_encode(Point value)"));
    assert!(c_source.contains("int32_t encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "make_point");
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_generic_function_return_enum_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Option_i32"));
    assert!(c_source.contains("Option_i32 wrap_i32(int32_t value)"));
    assert!(c_source.contains("int32_t unwrap_i32(Option_i32 value, int32_t fallback)"));
    assert!(c_source.contains("wrap_i32(107LL)"));
    assert!(c_source.contains("unwrap_i32(value, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "wrap_i32");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_i32");
    assert!(!c_source.contains("T wrap"));
    assert!(!c_source.contains("T unwrap"));
}
