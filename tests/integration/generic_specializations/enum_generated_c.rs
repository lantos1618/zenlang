use super::*;

#[test]
fn enum_specializations_do_not_emit_unspecialized_c_symbols() {
    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_enum_option.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("int32_t unwrap_or_i32(Option_i32 value, int32_t fallback)"));
    assert!(c_source.contains("Option_i32_Some"));
    assert!(c_source.contains("unwrap_or_i32(x, 0LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "unwrap_or_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T unwrap_or"));
    assert!(!c_source.contains("unwrap_or(x"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_enum_method.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("int32_t Option_unwrap_or_i32(Option_i32 self, int32_t fallback)"));
    assert!(c_source.contains("Option_unwrap_or_i32(some, 0LL)"));
    assert!(c_source.contains("Option_unwrap_or_i32(none, 55LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Option_unwrap_or_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T Option_unwrap_or"));
    assert!(!c_source.contains("Option_unwrap_or(some"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("generic_enum_multi_specialization.zen"),
    );
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("typedef struct Option_bool Option_bool;"));
    assert!(c_source.contains("int32_t Option_unwrap_or_i32(Option_i32 self, int32_t fallback)"));
    assert!(c_source.contains("bool Option_unwrap_or_bool(Option_bool self, bool fallback)"));
    assert!(c_source.contains("Option_unwrap_or_i32(some_int, 0LL)"));
    assert!(c_source.contains("Option_unwrap_or_i32(none_int, 55LL)"));
    assert!(c_source.contains("Option_unwrap_or_bool(some_bool, false)"));
    assert!(c_source.contains("Option_unwrap_or_bool(none_bool, true)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Option_unwrap_or_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "Option_unwrap_or_bool");
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
    assert!(c_source.contains("typedef struct Result_i32_StaticString Result_i32_StaticString;"));
    assert!(c_source.contains(
        "int32_t unwrap_or_i32_StaticString(Result_i32_StaticString value, int32_t fallback)"
    ));
    assert!(c_source.contains("Result_i32_StaticString_Err"));
    assert!(c_source.contains("unwrap_or_i32_StaticString(err, 9LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "unwrap_or_i32_StaticString");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T unwrap_or"));
    assert!(!c_source.contains("unwrap_or(err"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_result_enum_method.zen"));
    assert!(c_source.contains("typedef struct Result_i32_StaticString Result_i32_StaticString;"));
    assert!(c_source.contains(
        "int32_t Result_unwrap_or_i32_StaticString(Result_i32_StaticString self, int32_t fallback)"
    ));
    assert!(c_source.contains("Result_unwrap_or_i32_StaticString(ok, 0LL)"));
    assert!(c_source.contains("Result_unwrap_or_i32_StaticString(err, 34LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Result_unwrap_or_i32_StaticString");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T Result_unwrap_or"));
    assert!(!c_source.contains("Result_unwrap_or(err"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("generic_result_enum_multi_specialization.zen"),
    );
    assert!(c_source.contains("typedef struct Result_i32_StaticString Result_i32_StaticString;"));
    assert!(c_source.contains("typedef struct Result_bool_StaticString Result_bool_StaticString;"));
    assert!(c_source.contains(
        "int32_t Result_unwrap_or_i32_StaticString(Result_i32_StaticString self, int32_t fallback)"
    ));
    assert!(c_source.contains(
        "bool Result_unwrap_or_bool_StaticString(Result_bool_StaticString self, bool fallback)"
    ));
    assert!(c_source.contains("Result_unwrap_or_i32_StaticString(ok_int, 0LL)"));
    assert!(c_source.contains("Result_unwrap_or_i32_StaticString(err_int, 34LL)"));
    assert!(c_source.contains("Result_unwrap_or_bool_StaticString(ok_bool, true)"));
    assert!(c_source.contains("Result_unwrap_or_bool_StaticString(err_bool, true)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Result_unwrap_or_i32_StaticString");
    assert_c_call_resolves_to_single_definition(&c_source, "Result_unwrap_or_bool_StaticString");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T Result_unwrap_or"));
    assert!(!c_source.contains("Result_unwrap_or(err"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_nested_result_enum.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source
        .contains("typedef struct Result_Option_i32_StaticString Result_Option_i32_StaticString;"));
    assert!(c_source.contains(
        "Option_i32 unwrap_result_Option_i32_StaticString(Result_Option_i32_StaticString value, Option_i32 fallback)"
    ));
    assert!(c_source.contains("unwrap_result_Option_i32_StaticString(ok,"));
    assert!(c_source.contains("unwrap_option_i32(some, 0LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "unwrap_result_Option_i32_StaticString");
    assert_c_call_resolves_to_single_definition(&c_source, "unwrap_option_i32");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T unwrap_result"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("generic_enum_method_nested_result.zen"),
    );
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source
        .contains("typedef struct Result_Option_i32_StaticString Result_Option_i32_StaticString;"));
    assert!(
        c_source.contains("Result_Option_i32_StaticString Option_wrap_result_i32(Option_i32 self)")
    );
    assert!(c_source.contains("Option_wrap_result_i32(some)"));
    assert!(c_source.contains("Option_wrap_result_i32(none)"));
    assert!(c_source.contains("unwrap_result_Option_i32_StaticString(wrapped_some,"));
    assert_c_call_resolves_to_single_definition(&c_source, "Option_wrap_result_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "unwrap_result_Option_i32_StaticString");
    assert_c_call_resolves_to_single_definition(&c_source, "unwrap_option_i32");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T Option_wrap_result"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("generic_enum_nested_payload_inference.zen"),
    );
    assert!(c_source.contains("typedef struct Box_i32 Box_i32;"));
    assert!(c_source.contains("typedef struct Box_bool Box_bool;"));
    assert!(c_source.contains("typedef struct Choice_i32_bool Choice_i32_bool;"));
    assert!(c_source.contains("Choice_i32_bool pick_left_i32_bool(int32_t value)"));
    assert!(
        c_source.contains("int32_t unwrap_left_i32_bool(Choice_i32_bool choice, int32_t fallback)")
    );
    assert!(c_source.contains("unwrap_left_i32_bool(choice, 0LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "pick_left_i32_bool");
    assert_c_call_resolves_to_single_definition(&c_source, "unwrap_left_i32_bool");
    assert!(!c_source.contains("Box_T"));
    assert!(!c_source.contains("Choice_T"));
    assert!(!c_source.contains("T unwrap_left"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("generic_enum_method_self_renamed_params.zen"),
    );
    assert!(c_source.contains("typedef struct Choice_i32_bool Choice_i32_bool;"));
    assert!(c_source
        .contains("int32_t Choice_left_or_i32_bool(Choice_i32_bool self, int32_t fallback)"));
    assert!(c_source.contains("Choice_left_or_i32_bool(choice, 0LL)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Choice_left_or_i32_bool");
    assert!(!c_source.contains("void /* unknown */ self"));
    assert!(!c_source.contains("Choice_T"));
    assert!(!c_source.contains("A Choice_left_or"));
}
