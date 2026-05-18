use super::super::*;

#[test]
fn imported_function_behavior_bound_dependencies_do_not_emit_unspecialized_c_symbols() {
    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_imported_behavior_bound/main.zen"),
    );
    assert!(c_source.contains("int32_t Point_encode__Json_i32(Point value)"));
    assert!(c_source.contains("int32_t encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode__Json_i32(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode__Json_i32");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));
}

#[test]
fn imported_function_signature_type_dependencies_do_not_emit_unspecialized_c_symbols() {
    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_return_type_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Point"));
    assert!(c_source.contains("Point make_point(void)"));
    assert!(c_source.contains("int32_t Point_encode__Json_i32(Point value)"));
    assert!(c_source.contains("int32_t encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode__Json_i32(value)"));
    assert_c_call_resolves_to_definition(&c_source, "make_point");
    assert_c_call_resolves_to_definition(&c_source, "Point_encode__Json_i32");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_param_type_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Point"));
    assert!(c_source.contains("Point make_point(void)"));
    assert!(c_source.contains("int32_t Point_encode__Json_i32(Point value)"));
    assert!(c_source.contains("int32_t encode_point(Point value)"));
    assert!(c_source.contains("Point_encode__Json_i32(value)"));
    assert!(c_source.contains("encode_point(point)"));
    assert_c_call_resolves_to_definition(&c_source, "make_point");
    assert_c_call_resolves_to_definition(&c_source, "Point_encode__Json_i32");
    assert_c_call_resolves_to_definition(&c_source, "encode_point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_imported_return_type_behavior/main.zen"),
    );
    assert!(c_source.contains("typedef struct Point"));
    assert!(c_source.contains("Point make_point(void)"));
    assert!(c_source.contains("int32_t Point_encode__Json_i32(Point value)"));
    assert!(c_source.contains("int32_t encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode__Json_i32(value)"));
    assert_c_call_resolves_to_definition(&c_source, "make_point");
    assert_c_call_resolves_to_definition(&c_source, "Point_encode__Json_i32");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));
}
