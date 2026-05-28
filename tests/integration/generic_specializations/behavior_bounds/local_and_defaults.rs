use super::super::*;

#[test]
fn local_behavior_bound_specializations_do_not_emit_unspecialized_c_symbols() {
    assert_fixture_specialization(
        "behavior_json_generic_bound_ufcs.zen",
        &[
            "Point Point_encode__Json_Point(Point value)",
            "Point_encode__Json_Point(value)",
        ],
        &["Point_encode__Json_Point"],
        &["T_encode"],
    );

    assert_fixture_specialization(
        "behavior_generic_default_method.zen",
        &["zen_str Point_encode(Point __arg0)", "Point_encode(point)"],
        &["Point_encode"],
        &["Json_T", "T Point_encode"],
    );

    assert_fixture_specialization(
        "behavior_distinct_generic_specialization_dispatch.zen",
        &[
            "zen_str Point_encode__Json_StaticString(Point value)",
            "int32_t Point_encode__Json_i32(Point value)",
            "zen_str encode_str_Point(Point value)",
            "int32_t encode_i32_Point(Point value)",
            "Point_encode__Json_StaticString(value)",
            "Point_encode__Json_i32(value)",
        ],
        &[
            "Point_encode__Json_StaticString",
            "Point_encode__Json_i32",
            "encode_str_Point",
            "encode_i32_Point",
        ],
        &["Json_T", "T_encode"],
    );

    assert_fixture_specialization(
        "behavior_generic_target_association.zen",
        &[
            "int32_t Box_encode__Json_i32(Box_i32 self)",
            "bool Box_encode__Json_bool(Box_bool self)",
            "Box_encode__Json_i32(int_box)",
            "Box_encode__Json_bool(bool_box)",
        ],
        &["Box_encode__Json_i32", "Box_encode__Json_bool"],
        &["Box_encode__Json_T", "Json_T", "T_encode"],
    );

    assert_fixture_specialization(
        "behavior_generic_target_distinct_behavior_args.zen",
        &[
            "zen_str Box_encode__Json_StaticString_bool(Box_bool self)",
            "int32_t Box_encode__Json_i32_i32(Box_i32 self)",
            "Box_encode__Json_StaticString_bool(value)",
            "Box_encode__Json_i32_i32(value)",
        ],
        &[
            "Box_encode__Json_StaticString_bool",
            "Box_encode__Json_i32_i32",
        ],
        &[
            "Box_encode__Json_StaticString_i32",
            "Box_encode__Json_i32_bool",
            "Box_encode__Json_T",
            "Json_T",
        ],
    );

    assert_fixture_specialization(
        "behavior_generic_target_default_method.zen",
        &[
            "zen_str Box_describe_i32(Box_i32 self)",
            "zen_str Box_describe_bool(Box_bool self)",
            "Box_describe_i32(int_box)",
            "Box_describe_bool(bool_box)",
        ],
        &["Box_describe_i32", "Box_describe_bool"],
        &["Box_describe_T", "Describe_T"],
    );
}

#[test]
fn multi_file_behavior_bound_specializations_do_not_emit_unspecialized_c_symbols() {
    assert_fixture_specialization(
        "multi_file_behavior_bound/main.zen",
        &[
            "Point Point_encode__Json_Point(Point value)",
            "Point encode_Point(Point value)",
            "Point_encode__Json_Point(value)",
        ],
        &["Point_encode__Json_Point", "encode_Point"],
        &["T_encode"],
    );

    assert_point_encode_dispatch("multi_file_behavior_inheritance/main.zen");
}
