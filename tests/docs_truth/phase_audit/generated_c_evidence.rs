use super::super::*;

#[test]
fn nested_generic_result_generated_c_pins_definition_counts() {
    let enum_generated_c = read("tests/integration/generic_specializations/enum_generated_c.rs");
    let nested_result_block =
        generated_c_fixture_block(&enum_generated_c, "generic_nested_result_enum.zen");

    assert_specialization_call_names_pinned(
        nested_result_block,
        &["unwrap_result_Option_i32_StaticString", "unwrap_option_i32"],
        "nested generic Result<Option<T>, E> generated-C tests",
    );
}

#[test]
fn multi_file_nested_generic_method_generated_c_pins_definition_counts() {
    let method_worklist =
        read("tests/integration/generic_specializations/multifile_generated_c/method_worklist_dependencies.rs");
    let nested_method_block = generated_c_fixture_block(
        &method_worklist,
        "multi_file_type_method_nested_result_dependency/main.zen",
    );

    assert_specialization_call_names_pinned(
        nested_method_block,
        &[
            "Box_wrap_result_i32",
            "unwrap_result_Option_i32_StaticString",
            "unwrap_option_i32",
        ],
        "multi-file nested generic method generated-C tests",
    );
}

#[test]
fn local_nested_generic_method_generated_c_pins_definition_counts() {
    let method_worklist =
        read("tests/integration/generic_specializations/method_worklist_generated_c.rs");
    let nested_method_block =
        generated_c_fixture_block(&method_worklist, "generic_method_nested_result.zen");

    assert_specialization_call_names_pinned(
        nested_method_block,
        &[
            "Box_wrap_result_i32",
            "unwrap_result_Option_i32_StaticString",
            "unwrap_option_i32",
        ],
        "local nested generic method generated-C tests",
    );
}

#[test]
fn imported_transitive_worklist_generated_c_pins_definition_counts() {
    let method_worklist =
        read("tests/integration/generic_specializations/multifile_generated_c/method_worklist_dependencies.rs");
    let transitive_block = generated_c_fixture_block(
        &method_worklist,
        "multi_file_generic_imported_transitive_dependency/main.zen",
    );

    assert_specialization_call_names_pinned(
        transitive_block,
        &["inner_i32", "middle_i32", "outer_i32"],
        "imported transitive generic worklist tests",
    );
}

#[test]
fn scoped_imported_generic_ufc_generated_c_pins_recovery_evidence() {
    let scoped_type_inference = read(
        "tests/integration/generic_specializations/multifile_generated_c/scoped_type_inference.rs",
    );

    for required in [
        "multi_file_generic_imported_scoped_type_inference/main.zen",
        r#"typedef struct right_Box_i32 right_Box_i32;"#,
        r#"typedef struct Holder_right_Box_i32 Holder_right_Box_i32;"#,
        r#"int32_t take_box_i32(right_Box_i32 box)"#,
        r#"int32_t Box_extra_i32(right_Box_i32 self)"#,
        r#"int32_t Holder_extra_right_Box_i32(Holder_right_Box_i32 self)"#,
        r#"assert_c_call_resolves_to_single_definition(&c_source, "take_box_i32")"#,
        r#"assert_c_call_resolves_to_single_definition(&c_source, "Box_extra_i32")"#,
        r#"assert_c_call_resolves_to_single_definition(&c_source, "Holder_extra_right_Box_i32")"#,
    ] {
        assert!(
            scoped_type_inference.contains(required),
            "scoped imported generic UFC generated-C proof should pin recovery evidence: {required}"
        );
    }
}

fn assert_specialization_call_names_pinned(block: &str, call_names: &[&str], label: &str) {
    assert!(
        block.contains("compile_to_c_with_specialization_check(")
            || block.contains("assert_fixture_specialization("),
        "{label} should use the generated-C specialization facade"
    );

    for call_name in call_names {
        assert!(
            block.contains(&format!(r#""{call_name}""#)),
            "{label} should pin exact definition counts through specialization call list: {call_name}"
        );
    }
}
