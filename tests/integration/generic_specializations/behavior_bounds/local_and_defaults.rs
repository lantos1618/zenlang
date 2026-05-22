use super::super::*;

#[test]
fn local_behavior_bound_specializations_do_not_emit_unspecialized_c_symbols() {
    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("behavior_json_generic_bound_ufcs.zen"),
    );
    assert!(c_source.contains("Point Point_encode__Json_Point(Point value)"));
    assert!(c_source.contains("Point_encode__Json_Point(value)"));
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("behavior_generic_default_method.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point __arg0)"));
    assert!(c_source.contains("Point_encode(point)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Point_encode");
    assert!(!c_source.contains("Json_T"));
    assert!(!c_source.contains("T Point_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("behavior_distinct_generic_specialization_dispatch.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode__Json_StaticString(Point value)"));
    assert!(c_source.contains("int32_t Point_encode__Json_i32(Point value)"));
    assert!(c_source.contains("zen_str encode_str_Point(Point value)"));
    assert!(c_source.contains("int32_t encode_i32_Point(Point value)"));
    assert!(c_source.contains("Point_encode__Json_StaticString(value)"));
    assert!(c_source.contains("Point_encode__Json_i32(value)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Point_encode__Json_StaticString");
    assert_c_call_resolves_to_single_definition(&c_source, "Point_encode__Json_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "encode_str_Point");
    assert_c_call_resolves_to_single_definition(&c_source, "encode_i32_Point");
    assert!(!c_source.contains("Json_T"));
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("behavior_generic_target_association.zen"),
    );
    assert!(c_source.contains("int32_t Box_encode__Json_i32(Box_i32 self)"));
    assert!(c_source.contains("bool Box_encode__Json_bool(Box_bool self)"));
    assert!(c_source.contains("Box_encode__Json_i32(int_box)"));
    assert!(c_source.contains("Box_encode__Json_bool(bool_box)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Box_encode__Json_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "Box_encode__Json_bool");
    assert!(!c_source.contains("Box_encode__Json_T"));
    assert!(!c_source.contains("Json_T"));
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("behavior_generic_target_distinct_behavior_args.zen"),
    );
    assert!(c_source.contains("zen_str Box_encode__Json_StaticString_bool(Box_bool self)"));
    assert!(c_source.contains("int32_t Box_encode__Json_i32_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_encode__Json_StaticString_bool(value)"));
    assert!(c_source.contains("Box_encode__Json_i32_i32(value)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Box_encode__Json_StaticString_bool");
    assert_c_call_resolves_to_single_definition(&c_source, "Box_encode__Json_i32_i32");
    assert!(!c_source.contains("Box_encode__Json_StaticString_i32"));
    assert!(!c_source.contains("Box_encode__Json_i32_bool"));
    assert!(!c_source.contains("Box_encode__Json_T"));
    assert!(!c_source.contains("Json_T"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("behavior_generic_target_default_method.zen"),
    );
    assert!(c_source.contains("zen_str Box_describe_i32(Box_i32 self)"));
    assert!(c_source.contains("zen_str Box_describe_bool(Box_bool self)"));
    assert!(c_source.contains("Box_describe_i32(int_box)"));
    assert!(c_source.contains("Box_describe_bool(bool_box)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Box_describe_i32");
    assert_c_call_resolves_to_single_definition(&c_source, "Box_describe_bool");
    assert!(!c_source.contains("Box_describe_T"));
    assert!(!c_source.contains("Describe_T"));
}

#[test]
fn multi_file_behavior_bound_specializations_do_not_emit_unspecialized_c_symbols() {
    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_behavior_bound/main.zen"),
    );
    assert!(c_source.contains("Point Point_encode__Json_Point(Point value)"));
    assert!(c_source.contains("Point encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode__Json_Point(value)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Point_encode__Json_Point");
    assert_c_call_resolves_to_single_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_behavior_inheritance/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_single_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_single_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));
}
