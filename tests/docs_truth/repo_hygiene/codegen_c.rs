use super::*;

#[test]
fn codegen_c_function_emission_lives_in_focused_helper() {
    let types = read("src/codegen/c/types.rs");
    let functions = read("src/codegen/c/functions.rs");
    let c_mod = read("src/codegen/c/mod.rs");

    for helper in [
        "emit_function_forward_decl",
        "emit_function",
        "format_params",
        "emit_global",
    ] {
        assert!(
            !types.contains(&format!("fn {helper}")),
            "C type emission should not own function/global helper: {helper}"
        );
        assert!(
            functions.contains(&format!("fn {helper}")),
            "C function/global emission should live in focused helper: {helper}"
        );
    }

    assert!(
        c_mod.contains("mod functions;"),
        "C codegen should load focused function/global emission helper"
    );
}

#[test]
fn codegen_c_expression_operator_spelling_lives_in_focused_helper() {
    let emit = read("src/codegen/c/emit.rs");
    let operators = read("src/codegen/c/operators.rs");
    let literals = read("src/codegen/c/literals.rs");
    let c_mod = read("src/codegen/c/mod.rs");

    assert!(
        emit.lines().count() < 240,
        "C expression emission should stay focused on expression routing"
    );
    for helper in ["fn c_binary_op", "fn c_unary_op"] {
        assert!(
            !emit.contains(helper),
            "C expression emitter should not own operator spelling helper: {helper}"
        );
        assert!(
            operators.contains(helper),
            "C operator spelling should live in focused helper: {helper}"
        );
    }
    assert!(
        c_mod.contains("mod operators;"),
        "C codegen should load focused operator spelling helper"
    );
    for helper in [
        "emit_struct_literal",
        "emit_enum_variant_literal",
        "emit_array_literal",
    ] {
        assert!(
            !emit.contains(&format!("fn {helper}")),
            "C expression emitter should route aggregate literals to focused helper: {helper}"
        );
        assert!(
            literals.contains(&format!("fn {helper}")),
            "C aggregate literal emission should live in focused helper: {helper}"
        );
    }
    assert!(
        c_mod.contains("mod literals;"),
        "C codegen should load focused aggregate literal helper"
    );
}

#[test]
fn generated_c_tests_use_single_definition_assertion_helper() {
    let support = read("tests/integration/support.rs");
    let generated_c = read("tests/integration/support/generated_c.rs");
    let local_worklist =
        read("tests/integration/generic_specializations/method_worklist_generated_c.rs");
    let multi_file_worklist =
        read("tests/integration/generic_specializations/multifile_generated_c/method_worklist_dependencies.rs");
    let enum_generated_c = read("tests/integration/generic_specializations/enum_generated_c.rs");
    let behavior_bounds =
        read("tests/integration/generic_specializations/behavior_bounds/local_and_defaults.rs");
    let multi_file_enums = read(
        "tests/integration/generic_specializations/multifile_generated_c/enum_dependencies.rs",
    );

    assert!(
        generated_c.contains("fn assert_c_call_resolves_to_single_definition("),
        "generated-C support should expose a helper for call resolution plus exactly-one definition"
    );
    assert!(
        support.contains("assert_c_call_resolves_to_single_definition"),
        "integration support should re-export the generated-C single-definition helper"
    );

    for (fixture, helper_call) in [
        (
            local_worklist.as_str(),
            r#"assert_c_call_resolves_to_single_definition(&c_source, "Box_wrap_result_i32")"#,
        ),
        (
            local_worklist.as_str(),
            r#"assert_c_call_resolves_to_single_definition(&c_source, "unwrap_result_Option_i32_StaticString")"#,
        ),
        (
            multi_file_worklist.as_str(),
            r#"assert_c_call_resolves_to_single_definition(&c_source, "inner_i32")"#,
        ),
        (
            multi_file_worklist.as_str(),
            r#"assert_c_call_resolves_to_single_definition(&c_source, "outer_i32")"#,
        ),
    ] {
        assert!(
            fixture.contains(helper_call),
            "Phase 5 generated-C evidence should use the single-definition helper: {helper_call}"
        );
    }

    for source in [enum_generated_c, behavior_bounds, multi_file_enums] {
        assert!(
            !source.contains("assert_c_function_definition_count"),
            "Phase 5 generated-C tests should use the single-definition helper instead of split call/count assertions"
        );
    }
}
