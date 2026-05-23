use super::*;

#[test]
fn method_and_worklist_specializations_do_not_emit_unspecialized_c_symbols() {
    compile_to_c_with_specialization_check(
        &test_dir().join("generic_method.zen"),
        &["int32_t Box_get_i32(Box_i32 self)", "Box_get_i32(box)"],
        &["Box_get_i32"],
        &["Box_T", "T Box_get"],
    );

    let c_source = compile_to_c_with_specialization_check(
        &test_dir().join("generic_method_self.zen"),
        &[
            "Box_i32 Box_copy_i32(Box_i32 self)",
            "Box_copy_i32(box)",
            "Box_Option_i32 Box_copy_Option_i32(Box_Option_i32 self)",
            "Box_copy_Option_i32(nested)",
            "Option_i32 Option_copy_i32(Option_i32 self)",
            "Option_copy_i32(option)",
        ],
        &["Box_copy_i32", "Box_copy_Option_i32", "Option_copy_i32"],
        &["Box_copy(box", "Unknown"],
    );
    assert!(
        c_source
            .find("struct Option_i32")
            .expect("Option_i32 struct")
            < c_source
                .find("struct Box_Option_i32")
                .expect("Box_Option_i32 struct")
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_method_self_phantom.zen"),
        &[
            "Marker_i32 Marker_copy_i32(Marker_i32 self)",
            "Token_i32 Token_copy_i32(Token_i32 self)",
            "Marker_copy_i32(value)",
            "Token_copy_i32(token)",
        ],
        &["Marker_copy_i32", "Token_copy_i32"],
        &["Marker_T", "Token_T"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_method_self_param_order.zen"),
        &[
            "int32_t Box_tag_StaticString_i32(Box_i32 self, zen_str label)",
            "Box_tag_StaticString_i32(box,",
        ],
        &["Box_tag_StaticString_i32"],
        &["Box_tag_i32_StaticString"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_method_worklist.zen"),
        &[
            "int32_t inner_i32(int32_t value)",
            "int32_t Box_get_inner_i32(Box_i32 self)",
            "inner_i32(self.value)",
            "Box_get_inner_i32(box)",
        ],
        &["inner_i32", "Box_get_inner_i32"],
        &["T inner", "inner_T"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_method_method_worklist.zen"),
        &[
            "int32_t Box_inner_i32(Box_i32 self)",
            "int32_t Box_get_inner_i32(Box_i32 self)",
            "Box_inner_i32(self)",
            "Box_get_inner_i32(box)",
        ],
        &["Box_inner_i32", "Box_get_inner_i32"],
        &["T Box_inner"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_method_worklist_dedup.zen"),
        &[
            "int32_t Box_inner_i32(Box_i32 self)",
            "int32_t Box_left_inner_i32(Box_i32 self)",
            "int32_t Box_right_inner_i32(Box_i32 self)",
            "Box_inner_i32(self)",
            "Box_left_inner_i32(box)",
            "Box_right_inner_i32(box)",
        ],
        &["Box_inner_i32", "Box_left_inner_i32", "Box_right_inner_i32"],
        &["T Box_inner"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_method_nested_result.zen"),
        &[
            "typedef struct Result_Option_i32_StaticString Result_Option_i32_StaticString;",
            "Result_Option_i32_StaticString Box_wrap_result_i32(Box_i32 self)",
            "Box_wrap_result_i32(box)",
            "unwrap_result_Option_i32_StaticString(wrapped,",
            "unwrap_option_i32(some, 0LL)",
        ],
        &[
            "Box_wrap_result_i32",
            "unwrap_result_Option_i32_StaticString",
            "unwrap_option_i32",
        ],
        &["Result_T", "Option_T", "T Box_wrap_result"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("type_impl_methods.zen"),
        &[
            "int32_t Point_get(Point self)",
            "int32_t Point_keep_i32(Point self, int32_t value)",
            "Point_get(point)",
            "Point_keep_i32(point, 7LL)",
        ],
        &["Point_get", "Point_keep_i32"],
        &["T Point_keep", "Point_keep(point"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_type_impl_methods.zen"),
        &[
            "int32_t Box_get_i32(Box_i32 self)",
            "Box_i32 Box_replace_i32(Box_i32 self, int32_t value)",
            "Box_get_i32(next)",
            "Box_replace_i32(box, 42LL)",
        ],
        &["Box_get_i32", "Box_replace_i32"],
        &["Box_T", "T Box_get", "T Box_replace"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_vec.zen"),
        &[
            "int32_t Vec_len_i32(Vec_i32 self)",
            "int32_t Vec_len_StaticString(Vec_StaticString self)",
            "Vec_len_i32(ints)",
            "Vec_len_StaticString(words)",
        ],
        &["Vec_len_i32", "Vec_len_StaticString"],
        &["Vec_T", "T Vec_len"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_worklist.zen"),
        &[
            "int32_t inner_i32(int32_t value)",
            "int32_t outer_i32(int32_t value)",
            "inner_i32(value)",
        ],
        &["inner_i32", "outer_i32"],
        &["T inner", "inner_T"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_worklist_dedup.zen"),
        &[
            "int32_t left_i32(int32_t value)",
            "int32_t right_i32(int32_t value)",
            "inner_i32(value)",
        ],
        &["inner_i32", "left_i32", "right_i32"],
        &["T inner", "inner_T"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_recursive_function.zen"),
        &[
            "int32_t repeat_i32(int32_t value, int32_t remaining)",
            "repeat_i32(value, (remaining - 1LL))",
            "repeat_i32(41LL, 3LL)",
        ],
        &["repeat_i32"],
        &["T repeat", "repeat_T"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_recursive_method.zen"),
        &[
            "int32_t Box_repeat_i32(Box_i32 self, int32_t remaining)",
            "Box_repeat_i32(self, (remaining - 1LL))",
            "Box_repeat_i32(box, 3LL)",
        ],
        &["Box_repeat_i32"],
        &["T Box_repeat"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_ufc_function.zen"),
        &["int32_t id_i32(int32_t value)", "id_i32(12LL)"],
        &["id_i32"],
        &["id(12LL)", "T id"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("generic_ufc_dedup.zen"),
        &[
            "int32_t id_i32(int32_t value)",
            "id_i32(12LL)",
            "id_i32(30LL)",
        ],
        &["id_i32"],
        &["id(12LL)", "id(30LL)", "T id"],
    );
}
