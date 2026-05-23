use super::*;

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
    let scoped_type_inference = read(
        "tests/integration/generic_specializations/multifile_generated_c/scoped_type_inference.rs",
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
    assert!(
        imported_function_behavior_bounds.contains("compile_to_c_with_specialization_check("),
        "imported function behavior-bound generated-C evidence should compile through the grouped specialization facade"
    );
    assert!(
        !imported_function_behavior_bounds.contains("assert!(c_source.contains"),
        "imported function behavior-bound generated-C tests should group required snippets through the specialization facade"
    );
    assert!(
        !imported_function_behavior_bounds.contains("assert!(!c_source.contains"),
        "imported function behavior-bound generated-C tests should group forbidden snippets through the specialization facade"
    );
    assert!(
        scoped_type_inference.contains("compile_to_c_with_specialization_check("),
        "scoped type-inference generated-C evidence should compile through the grouped specialization facade"
    );
    assert!(
        !scoped_type_inference.contains("assert!(c_source.contains"),
        "scoped type-inference generated-C tests should group required snippets through the specialization facade"
    );
    assert!(
        !scoped_type_inference.contains("assert!(!c_source.contains"),
        "scoped type-inference generated-C tests should group forbidden snippets through the specialization facade"
    );

    for source in [
        &enum_generated_c,
        &behavior_bounds,
        &multi_file_enums,
        &imported_function_behavior_bounds,
        &scoped_type_inference,
    ] {
        assert!(
            !source.contains("assert_c_function_definition_count"),
            "Phase 5 generated-C tests should use the single-definition helper instead of split call/count assertions"
        );
    }

    for source in [&enum_generated_c, &multi_file_enums, &scoped_type_inference] {
        assert!(
            !source.contains("assert_c_call_resolves_to_definition(&c_source"),
            "enum generated-C tests should use the single-definition helper for generated call checks"
        );
    }

    assert!(
        !imported_function_behavior_bounds
            .contains("assert_c_call_resolves_to_definition(&c_source"),
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
