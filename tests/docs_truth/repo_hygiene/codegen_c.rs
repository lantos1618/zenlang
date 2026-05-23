use super::*;

#[path = "codegen_c/generated_c_support.rs"]
mod generated_c_support;

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
        operators.contains("op.symbol()"),
        "C operator spelling should reuse AST operator symbols instead of repeating a C-only table"
    );
    for duplicated_table_entry in ["BinaryOp::Add =>", "UnaryOp::Neg =>"] {
        assert!(
            !operators.contains(duplicated_table_entry),
            "C operator helper should not duplicate AST operator spelling: {duplicated_table_entry}"
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
fn codegen_c_does_not_lower_gated_enum_payload_mutation() {
    let lowering = read("src/codegen/c/intrinsics.rs");
    let spelling = read("src/codegen/c/intrinsics/names/spelling.rs");
    let gated = read("src/typechecker/gated_intrinsics/spelling.rs");

    assert!(
        !lowering.contains("memcpy((uint8_t*)({}) + sizeof(int32_t), {}, 0)"),
        "C codegen should not keep a zero-byte set_payload lowering placeholder"
    );
    assert!(
        !spelling.contains(r#"SetPayload => "set_payload""#),
        "set_payload should not be listed as an available C backend intrinsic until layout sizes exist"
    );
    assert!(
        gated.contains(r#"SetPayload => "set_payload""#),
        "set_payload should stay visible as a gated typechecker intrinsic"
    );
}

#[test]
fn generated_c_tests_use_facade_assertion_helpers() {
    let support = read("tests/integration/support.rs");
    let generated_c = read("tests/integration/support/generated_c.rs");
    let local_worklist =
        read("tests/integration/generic_specializations/method_worklist_generated_c.rs");
    let multi_file_worklist =
        read("tests/integration/generic_specializations/multifile_generated_c/method_worklist_dependencies.rs");
    let enum_generated_c = read("tests/integration/generic_specializations/enum_generated_c.rs");
    let behavior_bounds =
        read("tests/integration/generic_specializations/behavior_bounds/local_and_defaults.rs");
    let imported_function_behavior_bounds = read(
        "tests/integration/generic_specializations/behavior_bounds/imported_function_dependencies.rs",
    );
    let imported_behavior_impls = read(
        "tests/integration/generic_specializations/behavior_bounds/imported_behavior_impls.rs",
    );
    let multi_file_enums = read(
        "tests/integration/generic_specializations/multifile_generated_c/enum_dependencies.rs",
    );

    assert!(
        generated_c.contains("fn assert_c_call_resolves_to_single_definition("),
        "generated-C support should expose a helper for call resolution plus exactly-one definition"
    );
    assert!(
        generated_c.contains("fn assert_generated_c_specialization("),
        "generated-C support should expose a facade for grouped specialization assertions"
    );
    assert!(
        support.contains("assert_c_call_resolves_to_single_definition"),
        "integration support should re-export the generated-C single-definition helper"
    );
    assert!(
        support.contains("assert_generated_c_specialization"),
        "integration support should re-export the generated-C specialization facade"
    );
    assert!(
        !support.contains("assert_c_function_definition_count"),
        "integration support should not re-export split generated-C definition-count assertions"
    );
    assert!(
        !generated_c.contains("pub fn assert_c_call_resolves_to_definition("),
        "generated-C support should keep the weaker call-only helper private"
    );
    assert!(
        !generated_c.contains("pub fn assert_c_function_definition_count("),
        "generated-C support should keep the split definition-count helper private"
    );

    for fixture in [local_worklist.as_str(), multi_file_worklist.as_str()] {
        assert!(
            fixture.contains("compile_to_c_with_specialization_check("),
            "method/worklist generated-C evidence should compile through the grouped specialization facade"
        );
    }
    for fixture in [local_worklist.as_str(), multi_file_worklist.as_str()] {
        assert!(
            !fixture.contains("assert!(c_source.contains"),
            "method/worklist generated-C tests should group required snippets through the specialization facade"
        );
        assert!(
            !fixture.contains("assert!(!c_source.contains"),
            "method/worklist generated-C tests should group forbidden snippets through the specialization facade"
        );
    }
    assert!(
        enum_generated_c.contains("assert_fixture_specialization("),
        "enum generated-C evidence should compile through the grouped specialization facade"
    );
    assert!(
        !enum_generated_c.contains("assert!(c_source.contains"),
        "enum generated-C tests should group required snippets through the specialization facade"
    );
    assert!(
        !enum_generated_c.contains("assert!(!c_source.contains"),
        "enum generated-C tests should group forbidden snippets through the specialization facade"
    );
    assert!(
        multi_file_enums.contains("compile_to_c_with_specialization_check("),
        "multi-file enum generated-C evidence should compile through the grouped specialization facade"
    );
    assert!(
        !multi_file_enums.contains("assert!(c_source.contains"),
        "multi-file enum generated-C tests should group required snippets through the specialization facade"
    );
    assert!(
        !multi_file_enums.contains("assert!(!c_source.contains"),
        "multi-file enum generated-C tests should group forbidden snippets through the specialization facade"
    );

    for source in [&enum_generated_c, &behavior_bounds, &multi_file_enums] {
        assert!(
            !source.contains("assert_c_function_definition_count"),
            "Phase 5 generated-C tests should use the single-definition helper instead of split call/count assertions"
        );
    }

    for source in [&enum_generated_c, &multi_file_enums] {
        assert!(
            !source.contains("assert_c_call_resolves_to_definition(&c_source"),
            "enum generated-C tests should use the single-definition helper for generated call checks"
        );
    }

    assert!(
        !imported_function_behavior_bounds.contains("assert_c_call_resolves_to_definition(&c_source"),
        "imported function behavior-bound generated-C tests should use the single-definition helper for generated call checks"
    );
    assert!(
        !imported_behavior_impls.contains("assert_c_call_resolves_to_definition(&c_source"),
        "imported behavior-impl generated-C tests should use the single-definition helper for generated call checks"
    );
    assert!(
        !behavior_bounds.contains("assert_c_call_resolves_to_definition(&c_source"),
        "local behavior-bound generated-C tests should use the single-definition helper for generated call checks"
    );
    assert!(
        !local_worklist.contains("assert_c_call_resolves_to_definition(&c_source"),
        "local method/worklist generated-C tests should use the single-definition helper for generated call checks"
    );
    assert!(
        !multi_file_worklist.contains("assert_c_call_resolves_to_definition(&c_source"),
        "multi-file method/worklist generated-C tests should use the single-definition helper for generated call checks"
    );
}
