use super::super::*;

#[test]
fn imported_behavior_impl_specializations_do_not_emit_unspecialized_c_symbols() {
    assert_point_json_static_string_dispatch("multi_file_imported_behavior_impl/main.zen");

    assert_point_encode_dispatch("multi_file_imported_impl_imported_behavior/main.zen");

    assert_fixture_specialization(
        "multi_file_imported_generic_target_behavior_association/main.zen",
        &[
            "int32_t Box_encode__Json_i32(Box_i32 self)",
            "bool Box_encode__Json_bool(Box_bool self)",
            "Box_encode__Json_i32(value)",
            "Box_encode__Json_bool(value)",
        ],
        &["Box_encode__Json_i32", "Box_encode__Json_bool"],
        &["Box_encode__Json_T", "Json_T", "T_encode"],
    );
}

#[test]
fn imported_behavior_default_and_parent_dispatch_do_not_emit_unspecialized_c_symbols() {
    assert_fixture_specialization(
        "multi_file_imported_behavior_default/main.zen",
        &[
            "zen_str Point_to_json(Point ",
            "zen_str render_Point(Point value)",
            "Point_to_json(value)",
        ],
        &["Point_to_json", "render_Point"],
        &["T_to_json"],
    );

    assert_fixture_specialization(
        "multi_file_imported_generic_behavior_default/main.zen",
        &[
            "zen_str Point_encode(Point __arg0)",
            "zen_str render_Point(Point value)",
            "Point_encode(value)",
        ],
        &["Point_encode", "render_Point"],
        &["Json_T", "T_encode"],
    );

    assert_fixture_specialization(
        "multi_file_imported_child_parent_dispatch/main.zen",
        &[
            "zen_str Point_encode(Point value)",
            "zen_str render_Point(Point value)",
            "Point_encode(value)",
        ],
        &["Point_encode", "render_Point"],
        &["T_encode"],
    );

    assert_fixture_specialization(
        "multi_file_imported_generic_target_default_method/main.zen",
        &[
            "zen_str Box_describe_i32(Box_i32 self)",
            "zen_str Box_describe_bool(Box_bool self)",
            "Box_describe_i32(value)",
            "Box_describe_bool(value)",
        ],
        &["Box_describe_i32", "Box_describe_bool"],
        &["Box_describe_T", "Describe_T"],
    );
}

#[test]
fn imported_behavior_requires_do_not_emit_unspecialized_c_symbols() {
    assert_point_json_static_string_dispatch("multi_file_imported_behavior_requires/main.zen");

    assert_point_encode_dispatch("multi_file_imported_behavior_requires_inherited/main.zen");
}

fn assert_point_json_static_string_dispatch(fixture: &str) {
    assert_fixture_specialization(
        fixture,
        &[
            "zen_str Point_encode__Json_StaticString(Point value)",
            "zen_str encode_Point(Point value)",
            "Point_encode__Json_StaticString(value)",
        ],
        &["Point_encode__Json_StaticString", "encode_Point"],
        &["T_encode"],
    );
}
