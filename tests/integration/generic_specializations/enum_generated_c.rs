use super::*;

#[test]
fn enum_specializations_do_not_emit_unspecialized_c_symbols() {
    assert_fixture_specialization(
        "generic_enum_option.zen",
        &[
            "typedef struct Option_i32 Option_i32;",
            "int32_t unwrap_or_i32(Option_i32 value, int32_t fallback)",
            "Option_i32_Some",
            "unwrap_or_i32(x, 0LL)",
        ],
        &["unwrap_or_i32"],
        &["Option_T", "T unwrap_or", "unwrap_or(x"],
    );

    assert_fixture_specialization(
        "generic_enum_method.zen",
        &[
            "typedef struct Option_i32 Option_i32;",
            "int32_t Option_unwrap_or_i32(Option_i32 self, int32_t fallback)",
            "Option_unwrap_or_i32(some, 0LL)",
            "Option_unwrap_or_i32(none, 55LL)",
        ],
        &["Option_unwrap_or_i32"],
        &["Option_T", "T Option_unwrap_or", "Option_unwrap_or(some"],
    );

    assert_fixture_specialization(
        "generic_enum_multi_specialization.zen",
        &[
            "typedef struct Option_i32 Option_i32;",
            "typedef struct Option_bool Option_bool;",
            "int32_t Option_unwrap_or_i32(Option_i32 self, int32_t fallback)",
            "bool Option_unwrap_or_bool(Option_bool self, bool fallback)",
            "Option_unwrap_or_i32(some_int, 0LL)",
            "Option_unwrap_or_i32(none_int, 55LL)",
            "Option_unwrap_or_bool(some_bool, false)",
            "Option_unwrap_or_bool(none_bool, true)",
        ],
        &["Option_unwrap_or_i32", "Option_unwrap_or_bool"],
        &["Option_T", "T Option_unwrap_or", "Option_unwrap_or(some"],
    );

    assert_fixture_specialization(
        "duplicate_enum_variant_names.zen",
        &[
            "First_i32_Some",
            "First_i32_None",
            "Second_bool_Some",
            "Second_bool_None",
        ],
        &[],
        &[],
    );

    assert_fixture_specialization(
        "generic_result_enum.zen",
        &[
            "typedef struct Result_i32_StaticString Result_i32_StaticString;",
            "int32_t unwrap_or_i32_StaticString(Result_i32_StaticString value, int32_t fallback)",
            "Result_i32_StaticString_Err",
            "unwrap_or_i32_StaticString(err, 9LL)",
        ],
        &["unwrap_or_i32_StaticString"],
        &["Result_T", "T unwrap_or", "unwrap_or(err"],
    );

    assert_fixture_specialization(
        "generic_result_enum_method.zen",
        &[
            "typedef struct Result_i32_StaticString Result_i32_StaticString;",
            "int32_t Result_unwrap_or_i32_StaticString(Result_i32_StaticString self, int32_t fallback)",
            "Result_unwrap_or_i32_StaticString(ok, 0LL)",
            "Result_unwrap_or_i32_StaticString(err, 34LL)",
        ],
        &["Result_unwrap_or_i32_StaticString"],
        &["Result_T", "T Result_unwrap_or", "Result_unwrap_or(err"],
    );

    assert_fixture_specialization(
        "generic_result_enum_multi_specialization.zen",
        &[
            "typedef struct Result_i32_StaticString Result_i32_StaticString;",
            "typedef struct Result_bool_StaticString Result_bool_StaticString;",
            "int32_t Result_unwrap_or_i32_StaticString(Result_i32_StaticString self, int32_t fallback)",
            "bool Result_unwrap_or_bool_StaticString(Result_bool_StaticString self, bool fallback)",
            "Result_unwrap_or_i32_StaticString(ok_int, 0LL)",
            "Result_unwrap_or_i32_StaticString(err_int, 34LL)",
            "Result_unwrap_or_bool_StaticString(ok_bool, true)",
            "Result_unwrap_or_bool_StaticString(err_bool, true)",
        ],
        &[
            "Result_unwrap_or_i32_StaticString",
            "Result_unwrap_or_bool_StaticString",
        ],
        &["Result_T", "T Result_unwrap_or", "Result_unwrap_or(err"],
    );

    assert_fixture_specialization(
        "generic_nested_result_enum.zen",
        &[
            "typedef struct Option_i32 Option_i32;",
            "typedef struct Result_Option_i32_StaticString Result_Option_i32_StaticString;",
            "Option_i32 unwrap_result_Option_i32_StaticString(Result_Option_i32_StaticString value, Option_i32 fallback)",
            "unwrap_result_Option_i32_StaticString(ok,",
            "unwrap_option_i32(some, 0LL)",
        ],
        &[
            "unwrap_result_Option_i32_StaticString",
            "unwrap_option_i32",
        ],
        &["Result_T", "Option_T", "T unwrap_result"],
    );

    assert_fixture_specialization(
        "generic_enum_method_nested_result.zen",
        &[
            "typedef struct Option_i32 Option_i32;",
            "typedef struct Result_Option_i32_StaticString Result_Option_i32_StaticString;",
            "Result_Option_i32_StaticString Option_wrap_result_i32(Option_i32 self)",
            "Option_wrap_result_i32(some)",
            "Option_wrap_result_i32(none)",
            "unwrap_result_Option_i32_StaticString(wrapped_some,",
        ],
        &[
            "Option_wrap_result_i32",
            "unwrap_result_Option_i32_StaticString",
            "unwrap_option_i32",
        ],
        &["Result_T", "Option_T", "T Option_wrap_result"],
    );

    assert_fixture_specialization(
        "generic_enum_nested_payload_inference.zen",
        &[
            "typedef struct Box_i32 Box_i32;",
            "typedef struct Box_bool Box_bool;",
            "typedef struct Choice_i32_bool Choice_i32_bool;",
            "Choice_i32_bool pick_left_i32_bool(int32_t value)",
            "int32_t unwrap_left_i32_bool(Choice_i32_bool choice, int32_t fallback)",
            "unwrap_left_i32_bool(choice, 0LL)",
        ],
        &["pick_left_i32_bool", "unwrap_left_i32_bool"],
        &["Box_T", "Choice_T", "T unwrap_left"],
    );

    assert_fixture_specialization(
        "generic_enum_method_self_renamed_params.zen",
        &[
            "typedef struct Choice_i32_bool Choice_i32_bool;",
            "int32_t Choice_left_or_i32_bool(Choice_i32_bool self, int32_t fallback)",
            "Choice_left_or_i32_bool(choice, 0LL)",
        ],
        &["Choice_left_or_i32_bool"],
        &["void /* unknown */ self", "Choice_T", "A Choice_left_or"],
    );
}
