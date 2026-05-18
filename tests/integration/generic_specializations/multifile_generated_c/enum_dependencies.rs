use super::*;

#[test]
fn multi_file_generic_enum_specializations_do_not_emit_unspecialized_c_symbols() {
    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("multi_file_generic/main.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("typedef struct Result_i32_StaticString Result_i32_StaticString;"));
    assert!(c_source.contains("int32_t unwrap_option_i32(Option_i32 value, int32_t fallback)"));
    assert!(c_source.contains(
        "int32_t unwrap_result_i32_StaticString(Result_i32_StaticString value, int32_t fallback)"
    ));
    assert!(c_source.contains("unwrap_option_i32(some, 0LL)"));
    assert!(c_source.contains("unwrap_result_i32_StaticString(err, 9LL)"));
    assert_c_call_resolves_to_definition(&c_source, "unwrap_option_i32");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_result_i32_StaticString");
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
        &test_dir().join("multi_file_generic_result_enum_method/main.zen"),
    );
    assert!(c_source.contains("typedef struct Result_i32_StaticString Result_i32_StaticString;"));
    assert!(c_source.contains(
        "int32_t Result_unwrap_or_i32_StaticString(Result_i32_StaticString self, int32_t fallback)"
    ));
    assert!(c_source.contains("Result_unwrap_or_i32_StaticString(ok, 0LL)"));
    assert!(c_source.contains("Result_unwrap_or_i32_StaticString(err, 144LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Result_unwrap_or_i32_StaticString");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T Result_unwrap_or"));
    assert!(!c_source.contains("Result_unwrap_or(err"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_generic_result_enum_multi_specialization/main.zen"),
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
    assert!(c_source.contains("Result_unwrap_or_i32_StaticString(err_int, 144LL)"));
    assert!(c_source.contains("Result_unwrap_or_bool_StaticString(ok_bool, true)"));
    assert!(c_source.contains("Result_unwrap_or_bool_StaticString(err_bool, true)"));
    assert_c_call_resolves_to_definition(&c_source, "Result_unwrap_or_i32_StaticString");
    assert_c_call_resolves_to_definition(&c_source, "Result_unwrap_or_bool_StaticString");
    assert_c_function_definition_count(&c_source, "Result_unwrap_or_i32_StaticString", 1);
    assert_c_function_definition_count(&c_source, "Result_unwrap_or_bool_StaticString", 1);
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T Result_unwrap_or"));
    assert!(!c_source.contains("Result_unwrap_or(err"));

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
