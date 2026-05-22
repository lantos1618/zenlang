use super::*;

#[test]
fn single_file_fixture_tests_stay_split_by_fixture_family() {
    let root = read("tests/integration/single_file_fixtures.rs");
    let basics = read("tests/integration/single_file_fixtures/basics.rs");
    let behaviors = read("tests/integration/single_file_fixtures/behaviors.rs");
    let generics = read("tests/integration/single_file_fixtures/generics.rs");

    assert!(
        root.lines().count() < 60,
        "single_file_fixtures.rs should route focused single-file fixture modules"
    );
    assert!(
        !root.contains("#[test]"),
        "single_file_fixtures.rs should not own concrete fixture tests"
    );
    for module in [
        r#"#[path = "single_file_fixtures/basics.rs"]"#,
        r#"#[path = "single_file_fixtures/behaviors.rs"]"#,
        r#"#[path = "single_file_fixtures/generics.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "single_file_fixtures.rs should include focused module path `{module}`"
        );
    }

    assert_single_file_tests_live_in(
        &root,
        &basics,
        &[
            "test_hello",
            "test_arithmetic",
            "test_structs",
            "test_enums",
            "test_duplicate_enum_variant_names",
            "test_ufc",
            "test_conditionals",
            "test_loops",
            "test_strings",
            "test_functions",
            "test_type_impl_methods",
        ],
        "basics.rs",
    );
    assert_single_file_tests_live_in(
        &root,
        &generics,
        &[
            "test_generic_identity",
            "test_generic_struct",
            "test_generic_enum_option",
            "test_generic_enum_method",
            "test_generic_enum_multi_specialization",
            "test_generic_method",
            "test_generic_method_self",
            "test_generic_method_worklist",
            "test_generic_method_nested_result",
            "test_generic_type_impl_methods",
            "test_generic_result_enum",
            "test_generic_result_enum_method",
            "test_generic_result_enum_multi_specialization",
            "test_generic_nested_result_enum",
            "test_generic_vec",
            "test_generic_worklist",
            "test_generic_worklist_dedup",
            "test_generic_ufc_function",
            "test_generic_ufc_dedup",
        ],
        "generics.rs",
    );
    assert_single_file_tests_live_in(
        &root,
        &behaviors,
        &[
            "test_behavior_json_explicit_impl",
            "test_behavior_default_method_dispatch",
            "test_behavior_generic_default_method",
            "test_behavior_inherited_default_method",
            "test_behavior_json_generic_dispatch",
            "test_behavior_json_generic_association",
            "test_behavior_distinct_generic_specialization_dispatch",
            "test_behavior_json_generic_bound",
            "test_behavior_json_generic_bound_ufcs",
            "test_behavior_generic_parent_inheritance",
            "test_behavior_generic_parent_type_arg_inheritance",
            "test_behavior_inherited_generic_dispatch",
        ],
        "behaviors.rs",
    );
}

fn assert_single_file_tests_live_in(
    root: &str,
    focused_module: &str,
    test_names: &[&str],
    focused_path: &str,
) {
    for test_name in test_names {
        let fn_name = format!("fn {test_name}");
        assert!(
            !root.contains(&fn_name),
            "single-file fixture test should move out of the root module: {test_name}"
        );
        assert!(
            focused_module.contains(&fn_name),
            "{focused_path} should keep single-file fixture test: {test_name}"
        );
    }
}
