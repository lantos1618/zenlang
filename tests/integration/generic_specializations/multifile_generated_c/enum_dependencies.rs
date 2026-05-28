use super::*;

#[test]
fn multi_file_generic_enum_specializations_do_not_emit_unspecialized_c_symbols() {
    assert_fixture_specialization(
        "multi_file_generic/main.zen",
        &[
            "typedef struct Option_i32 Option_i32;",
            "typedef struct Result_i32_StaticString Result_i32_StaticString;",
            "int32_t unwrap_option_i32(Option_i32 value, int32_t fallback)",
            "int32_t unwrap_result_i32_StaticString(Result_i32_StaticString value, int32_t fallback)",
            "unwrap_option_i32(some, 0LL)",
            "unwrap_result_i32_StaticString(err, 9LL)",
        ],
        &["unwrap_option_i32", "unwrap_result_i32_StaticString"],
        &[
            "Option_T",
            "Result_T",
            "T unwrap_option",
            "T unwrap_result",
            "unwrap_option(some",
            "unwrap_result(err",
        ],
    );

    assert_option_unwrap_or_method(
        "multi_file_generic_enum_method/main.zen",
        "Option_unwrap_or_i32(none, 89LL)",
    );

    assert_result_unwrap_or_method(
        "multi_file_generic_result_enum_method/main.zen",
        "Result_unwrap_or_i32_StaticString(err, 144LL)",
    );

    assert_result_unwrap_or_multi_specialization(
        "multi_file_generic_result_enum_multi_specialization/main.zen",
        "Result_unwrap_or_i32_StaticString(err_int, 144LL)",
    );

    assert_fixture_specialization(
        "multi_file_imported_generic_function_return_enum_dependency/main.zen",
        &[
            "typedef struct Option_i32",
            "Option_i32 wrap_i32(int32_t value)",
            "int32_t unwrap_i32(Option_i32 value, int32_t fallback)",
            "wrap_i32(107LL)",
            "unwrap_i32(value, 0LL)",
        ],
        &["wrap_i32", "unwrap_i32"],
        &["T wrap", "T unwrap"],
    );

    assert_fixture_specialization(
        "multi_file_generic_imported_type_same_name_collision/main.zen",
        &[
            "left_i32(1LL)",
            "right_i32(2LL)",
            "typedef struct Box_i32 Box_i32;",
            "typedef struct right_Box_i32 right_Box_i32;",
            "typedef struct Choice_i32 Choice_i32;",
            "typedef struct right_Choice_i32 right_Choice_i32;",
            "const Box_i32 box = (Box_i32){ .value = value };",
            "const right_Box_i32 box = (right_Box_i32){ .value = value, .extra = 29LL };",
            "__tmp2 = 11LL;",
            "int32_t found = choice.data.extra;",
            "__tmp3 = found;",
        ],
        &["left_i32", "right_i32"],
        &["Box_T", "Choice_T"],
    );
}

#[test]
fn multi_file_generic_enum_method_worklist_specializations_emit_reachable_methods_once() {
    assert_fixture_specialization(
        "multi_file_generic_enum_method_worklist/main.zen",
        &[
            "typedef struct Option_i32 Option_i32;",
            "typedef struct Option_bool Option_bool;",
            "int32_t Option_value_or_i32(Option_i32 self, int32_t fallback)",
            "int32_t Option_unwrap_or_i32(Option_i32 self, int32_t fallback)",
            "bool Option_value_or_bool(Option_bool self, bool fallback)",
            "bool Option_unwrap_or_bool(Option_bool self, bool fallback)",
            "Option_value_or_i32(some_int, 0LL)",
            "Option_value_or_i32(none_int, 97LL)",
            "Option_value_or_bool(some_bool, false)",
            "Option_value_or_bool(none_bool, true)",
            "Option_unwrap_or_i32(self, fallback)",
            "Option_unwrap_or_bool(self, fallback)",
        ],
        &[
            "Option_value_or_i32",
            "Option_unwrap_or_i32",
            "Option_value_or_bool",
            "Option_unwrap_or_bool",
        ],
        &[
            "Option_T",
            "T Option_value_or",
            "T Option_unwrap_or",
            "Option_value_or(some",
            "Option_unwrap_or(self",
        ],
    );
}

#[test]
fn multi_file_generic_result_error_type_specializations_do_not_collapse() {
    assert_fixture_specialization(
        "multi_file_generic_result_error_multi_specialization/main.zen",
        &[
            "typedef struct Result_i32_bool Result_i32_bool;",
            "typedef struct Result_i32_i32 Result_i32_i32;",
            "bool Result_unwrap_err_i32_bool(Result_i32_bool self, bool fallback)",
            "int32_t Result_unwrap_err_i32_i32(Result_i32_i32 self, int32_t fallback)",
            "Result_unwrap_err_i32_bool(err_bool, false)",
            "Result_unwrap_err_i32_bool(ok_bool, false)",
            "Result_unwrap_err_i32_i32(err_i32, 0LL)",
            "Result_unwrap_err_i32_i32(ok_i32, 88LL)",
        ],
        &["Result_unwrap_err_i32_bool", "Result_unwrap_err_i32_i32"],
        &[
            "Result_T",
            "Result_i32_E",
            "E Result_unwrap_err",
            "Result_unwrap_err(err",
        ],
    );
}
