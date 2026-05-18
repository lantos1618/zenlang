use super::super::*;

#[test]
fn imported_behavior_impl_specializations_do_not_emit_unspecialized_c_symbols() {
    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_behavior_impl/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode__Json_StaticString(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode__Json_StaticString(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode__Json_StaticString");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_impl_imported_behavior/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));
}

#[test]
fn imported_behavior_default_and_parent_dispatch_do_not_emit_unspecialized_c_symbols() {
    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_behavior_default/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_to_json(Point "));
    assert!(c_source.contains("zen_str render_Point(Point value)"));
    assert!(c_source.contains("Point_to_json(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_to_json");
    assert_c_call_resolves_to_definition(&c_source, "render_Point");
    assert!(!c_source.contains("T_to_json"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_generic_behavior_default/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point __arg0)"));
    assert!(c_source.contains("zen_str render_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "render_Point");
    assert!(!c_source.contains("Json_T"));
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_child_parent_dispatch/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str render_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "render_Point");
    assert!(!c_source.contains("T_encode"));
}

#[test]
fn imported_behavior_requires_do_not_emit_unspecialized_c_symbols() {
    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_behavior_requires/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode__Json_StaticString(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode__Json_StaticString(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode__Json_StaticString");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_behavior_requires_inherited/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));
}
