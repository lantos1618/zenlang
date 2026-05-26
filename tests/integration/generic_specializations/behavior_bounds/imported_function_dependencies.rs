use super::super::*;

#[test]
fn imported_function_behavior_bound_dependencies_do_not_emit_unspecialized_c_symbols() {
    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_imported_function_imported_behavior_bound/main.zen"),
        &[
            "int32_t Point_encode__Json_i32(Point value)",
            "int32_t encode_Point(Point value)",
            "Point_encode__Json_i32(value)",
        ],
        &["Point_encode__Json_i32", "encode_Point"],
        &["T_encode"],
    );
}

#[test]
fn imported_function_signature_type_dependencies_do_not_emit_unspecialized_c_symbols() {
    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_imported_function_return_type_dependency/main.zen"),
        &[
            "typedef struct Point",
            "Point make_point(void)",
            "int32_t Point_encode__Json_i32(Point value)",
            "int32_t encode_Point(Point value)",
            "Point_encode__Json_i32(value)",
        ],
        &["make_point", "Point_encode__Json_i32", "encode_Point"],
        &["T_encode"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_imported_function_param_type_dependency/main.zen"),
        &[
            "typedef struct Point",
            "Point make_point(void)",
            "int32_t Point_encode__Json_i32(Point value)",
            "int32_t encode_point(Point value)",
            "Point_encode__Json_i32(value)",
            "encode_point(point)",
        ],
        &["make_point", "Point_encode__Json_i32", "encode_point"],
        &["T_encode"],
    );

    compile_to_c_with_specialization_check(
        &test_dir().join("multi_file_imported_function_imported_return_type_behavior/main.zen"),
        &[
            "typedef struct Point",
            "Point make_point(void)",
            "int32_t Point_encode__Json_i32(Point value)",
            "int32_t encode_Point(Point value)",
            "Point_encode__Json_i32(value)",
        ],
        &["make_point", "Point_encode__Json_i32", "encode_Point"],
        &["T_encode"],
    );
}
