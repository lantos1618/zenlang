use super::*;

#[test]
fn multi_file_generic_method_and_worklist_specializations_do_not_emit_unspecialized_c_symbols() {
    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_generic_imported_type_dependency/main.zen"),
        &[
            "typedef struct Holder_i32 Holder_i32;",
            "int32_t Holder_get_i32(Holder_i32 self)",
            "int32_t get_held_i32(int32_t value)",
            "const Holder_i32 holder = (Holder_i32){ .value = value }",
            "Holder_get_i32(holder)",
            "get_held_i32(73LL)",
        ],
        &["Holder_get_i32", "get_held_i32"],
        &["Holder_T", "T Holder_get"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_generic_imported_worklist_chain/main.zen"),
        &[
            "int32_t inner_i32(int32_t value)",
            "int32_t middle_i32(int32_t value)",
            "int32_t outer_i32(int32_t value)",
            "inner_i32(value)",
            "middle_i32(value)",
            "outer_i32(83LL)",
        ],
        &["inner_i32", "middle_i32", "outer_i32"],
        &["T inner", "T middle"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_generic_imported_worklist_multi_specialization/main.zen"),
        &[
            "int32_t inner_i32(int32_t value)",
            "bool inner_bool(bool value)",
            "int32_t middle_i32(int32_t value)",
            "bool middle_bool(bool value)",
            "int32_t outer_i32(int32_t value)",
            "bool outer_bool(bool value)",
            "outer_i32(83LL)",
            "outer_bool(true)",
        ],
        &[
            "inner_i32",
            "inner_bool",
            "middle_i32",
            "middle_bool",
            "outer_i32",
            "outer_bool",
        ],
        &["T inner", "T middle", "T outer"],
    );

    let c_source = compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_generic_imported_diamond_same_name/main.zen"),
        &[
            "int32_t left_i32(int32_t value)",
            "int32_t right_i32(int32_t value)",
            "return 11LL;",
            "return 29LL;",
            "left_i32(1LL)",
            "right_i32(2LL)",
        ],
        &["left_i32", "right_i32"],
        &["T inner"],
    );
    let left_inner = returned_call_name(&c_source, "left_i32");
    let right_inner = returned_call_name(&c_source, "right_i32");
    assert_eq!(left_inner, "inner_i32");
    assert_ne!(right_inner, "inner_i32");
    assert!(right_inner.ends_with("_inner_i32"));
    assert_c_call_resolves_to_single_definition(&c_source, &right_inner);

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_generic_recursive_function/main.zen"),
        &[
            "int32_t repeat_i32(int32_t value, int32_t remaining)",
            "repeat_i32(value, (remaining - 1LL))",
            "repeat_i32(97LL, 4LL)",
        ],
        &["repeat_i32"],
        &["T repeat", "repeat_T"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_generic_imported_transitive_dependency/main.zen"),
        &[
            "int32_t inner_i32(int32_t value)",
            "int32_t middle_i32(int32_t value)",
            "int32_t outer_i32(int32_t value)",
            "inner_i32(value)",
            "middle_i32(value)",
            "outer_i32(89LL)",
        ],
        &["inner_i32", "middle_i32", "outer_i32"],
        &["T inner", "T middle"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_type_impl/main.zen"),
        &["int32_t Box_get_i32(Box_i32 self)", "Box_get_i32(box)"],
        &["Box_get_i32"],
        &["Box_T", "T Box_get"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_type_impl_imported_type_dependency/main.zen"),
        &[
            "typedef struct Holder_i32 Holder_i32;",
            "int32_t Holder_get_i32(Holder_i32 self)",
            "int32_t Box_get_held_i32(Box_i32 self)",
            "const Holder_i32 holder = (Holder_i32){ .value = self.value }",
            "Holder_get_i32(holder)",
            "Box_get_held_i32(box)",
        ],
        &["Holder_get_i32", "Box_get_held_i32"],
        &["Holder_T", "T Holder_get"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_type_impl_return_enum_dependency/main.zen"),
        &[
            "typedef struct Option_i32 Option_i32;",
            "Option_i32 Box_wrap_i32(Box_i32 self)",
            "int32_t Box_value_or_i32(Box_i32 self, int32_t fallback)",
            "Box_wrap_i32(self)",
            "Box_value_or_i32(box, 0LL)",
        ],
        &["Box_wrap_i32", "Box_value_or_i32"],
        &["Option_T", "T Box_wrap"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_type_method/main.zen"),
        &[
            "int32_t Point_keep_i32(Point self, int32_t value)",
            "Point_keep_i32(point, 13LL)",
        ],
        &["Point_keep_i32"],
        &["T Point_keep", "Point_keep(point"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_type_method_worklist/main.zen"),
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
        &test_dir().join("multi_file_type_method_method_dependency/main.zen"),
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
        &test_dir().join("multi_file_type_method_imported_dependency/main.zen"),
        &[
            "int32_t inner_i32(int32_t value)",
            "int32_t Box_get_inner_i32(Box_i32 self)",
            "inner_i32(self.value)",
            "Box_get_inner_i32(box)",
        ],
        &["inner_i32", "Box_get_inner_i32"],
        &["T inner"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_type_method_return_enum_dependency/main.zen"),
        &[
            "typedef struct Option_i32 Option_i32;",
            "Option_i32 Box_wrap_i32(Box_i32 self)",
            "int32_t Box_value_or_i32(Box_i32 self, int32_t fallback)",
            "Box_wrap_i32(self)",
            "Box_value_or_i32(box, 0LL)",
        ],
        &["Box_wrap_i32", "Box_value_or_i32"],
        &["Option_T", "T Box_wrap"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_type_method_nested_result_dependency/main.zen"),
        &[
            "typedef struct Option_i32 Option_i32;",
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
        &["Option_T", "Result_Option_T", "T Box_wrap_result"],
    );
}

fn returned_call_name(c_source: &str, function_name: &str) -> String {
    let signature = format!(" {function_name}(");
    let mut in_function = false;
    for line in c_source.lines() {
        let trimmed = line.trim();
        if trimmed.ends_with('{') && trimmed.contains(&signature) {
            in_function = true;
            continue;
        }
        if !in_function {
            continue;
        }
        if trimmed == "}" {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("return ") {
            return rest
                .split('(')
                .next()
                .expect("return call should include function name")
                .to_string();
        }
    }
    panic!("expected return call in generated C function `{function_name}`:\n{c_source}");
}
