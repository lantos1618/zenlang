use super::*;

#[test]
fn enum_specializations_do_not_emit_unspecialized_c_symbols() {
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
    assert_c_call_resolves_to_definition(&c_source, "Option_unwrap_or_i32");
    assert_c_call_resolves_to_definition(&c_source, "Option_unwrap_or_bool");
    assert_c_function_definition_count(&c_source, "Option_unwrap_or_i32", 1);
    assert_c_function_definition_count(&c_source, "Option_unwrap_or_bool", 1);
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
        compile_to_c_with_generated_call_check(&test_dir().join("generic_result_enum_method.zen"));
    assert!(c_source.contains("typedef struct Result_i32_str Result_i32_str;"));
    assert!(c_source
        .contains("int32_t Result_unwrap_or_i32_str(Result_i32_str self, int32_t fallback)"));
    assert!(c_source.contains("Result_unwrap_or_i32_str(ok, 0LL)"));
    assert!(c_source.contains("Result_unwrap_or_i32_str(err, 34LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Result_unwrap_or_i32_str");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T Result_unwrap_or"));
    assert!(!c_source.contains("Result_unwrap_or(err"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("generic_result_enum_multi_specialization.zen"),
    );
    assert!(c_source.contains("typedef struct Result_i32_str Result_i32_str;"));
    assert!(c_source.contains("typedef struct Result_bool_str Result_bool_str;"));
    assert!(c_source
        .contains("int32_t Result_unwrap_or_i32_str(Result_i32_str self, int32_t fallback)"));
    assert!(
        c_source.contains("bool Result_unwrap_or_bool_str(Result_bool_str self, bool fallback)")
    );
    assert!(c_source.contains("Result_unwrap_or_i32_str(ok_int, 0LL)"));
    assert!(c_source.contains("Result_unwrap_or_i32_str(err_int, 34LL)"));
    assert!(c_source.contains("Result_unwrap_or_bool_str(ok_bool, true)"));
    assert!(c_source.contains("Result_unwrap_or_bool_str(err_bool, true)"));
    assert_c_call_resolves_to_definition(&c_source, "Result_unwrap_or_i32_str");
    assert_c_call_resolves_to_definition(&c_source, "Result_unwrap_or_bool_str");
    assert_c_function_definition_count(&c_source, "Result_unwrap_or_i32_str", 1);
    assert_c_function_definition_count(&c_source, "Result_unwrap_or_bool_str", 1);
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T Result_unwrap_or"));
    assert!(!c_source.contains("Result_unwrap_or(err"));

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
}
