use super::super::*;

#[test]
fn phase_plan_records_recovered_progress_and_next_slice() {
    let plan = read("docs/PHASE_PLAN.md");

    for required in [
        "Recovery Point",
        "183d140c",
        "Design Decisions To Preserve",
        "Dev UX And Agent UX Track",
        "Compiler And Stdlib Boundary",
        "Compressed Evidence Map",
        "Current Phase",
        "Phase 5 Acceptance Evidence",
        "generic enum specialization",
        "generic method specialization",
        "worklist monomorphization",
        "generated-C call/definition consistency",
        "generic arity, inference, and bound diagnostics",
        "Next Small Slice",
        "Detailed Evidence References",
        "Sync/Async are real effects",
        "typed allocators",
        "actors live in std first",
        "AST/HIR traversal is tooling/metaprogramming",
        "type matching and behavior association are separate",
        "JSON is compiler-owned IR output",
        "YAML is human-authored config/spec input",
        "build.zen is deterministic comptime build graph",
        "Project imports must resolve through the stdlib surface",
        "project root `build.zen`",
        "MoonBit-style toolchain integration",
        "VS Code extension",
        "language server",
        "agent-readable diagnostics",
        "Machine-readable project graph",
        "structured fix suggestions",
        "Compiler-owned: parsing, typing, resolver metadata",
        "primitive `@builtin` hooks",
        "Stdlib-owned: allocator implementations",
        "Dynamic `String` should be implemented in stdlib",
        "not as parser/compiler-owned special syntax",
        "Raw `@builtin` calls should stay behind `stdlib/compiler.zen`",
        "Most current stdlib files are placeholders/sketches",
        "real parser, typechecker, build path, and docs-truth gates",
        "Stdlib anti-slop pass",
        "stale LLVM-era wording",
        "Phase 0 truth gates",
        "Phase 1 frontend and C-backend baseline",
        "generic specialization",
        "resolver/typechecker replay",
        "build graph",
        "diagnostics JSON",
        "repo hygiene",
        "docs/V1_SPEC.md",
        "docs/DIAGNOSTICS.md",
        "docs/learn_zen_in_y_minutes.md",
        "tests/docs_truth",
        "tests/integration",
        "tests/resolver_phase2.rs",
        "tests/zen",
        "deterministic_build_graph_creates_one_executable_target",
        "emit_json_ast_marks_semantically_unchecked_sources_that_typed_json_rejects",
        "emit_json_typed_command_outputs_checked_program",
        "emit_json_diagnostics_command_outputs_machine_readable_errors",
        "emit_json_diagnostics_includes_structured_return_keyword_fix",
        "emit_json_diagnostics_includes_structured_missing_bool_match_arm_fix",
        "suggested_fixes",
        "feature_gate",
        "Type.implements(Behavior)",
        "non-generic explicit behavior associations",
        "generic_struct_constructor_without_type_args_is_error",
        "async_scheduler_intrinsics_are_rejected_as_gated_not_unknown",
        "atomic_intrinsics_are_rejected_as_effect_gates",
        "comptime_type_match_intrinsic_is_rejected_as_gated_not_unknown",
        "dynamic_string_type_is_rejected_as_allocator_backed_gate",
        "raw_memory_intrinsics_are_rejected_as_allocator_gates",
        "syscall_intrinsics_are_rejected_as_host_effect_gates",
        "StaticString",
        "allocator-backed `String`",
    ]
    .into_iter()
    {
        assert!(
            plan.contains(required),
            "docs/PHASE_PLAN.md is missing durable plan text: {required}"
        );
    }

    assert!(
        plan.lines().count() <= 100,
        "docs/PHASE_PLAN.md should stay compact; move granular evidence to tests or git history"
    );

    assert!(
        !plan.contains("Generic behavior inheritance in `.extends` is still explicitly gated"),
        "docs/PHASE_PLAN.md still claims generic behavior inheritance is gated"
    );
}

#[test]
fn nested_generic_result_generated_c_pins_definition_counts() {
    let enum_generated_c = read("tests/integration/generic_specializations/enum_generated_c.rs");
    let normalized_enum_generated_c = enum_generated_c.split_whitespace().collect::<String>();

    for required in [
        "assert_c_call_resolves_to_single_definition(&c_source,\"unwrap_result_Option_i32_StaticString\"",
        "assert_c_call_resolves_to_single_definition(&c_source,\"unwrap_option_i32\"",
    ] {
        assert!(
            normalized_enum_generated_c.contains(required),
            "nested generic Result<Option<T>, E> generated-C tests should pin exact definition counts: {required}"
        );
    }
}

#[test]
fn multi_file_nested_generic_method_generated_c_pins_definition_counts() {
    let method_worklist =
        read("tests/integration/generic_specializations/multifile_generated_c/method_worklist_dependencies.rs");
    let nested_method_block = generated_c_fixture_block(
        &method_worklist,
        "multi_file_type_method_nested_result_dependency/main.zen",
    );

    for required in [
        r#"assert_c_call_resolves_to_single_definition(&c_source, "Box_wrap_result_i32")"#,
        r#"assert_c_call_resolves_to_single_definition(&c_source, "unwrap_result_Option_i32_StaticString")"#,
        r#"assert_c_call_resolves_to_single_definition(&c_source, "unwrap_option_i32")"#,
    ] {
        assert!(
            nested_method_block.contains(required),
            "multi-file nested generic method generated-C tests should pin exact definition counts: {required}"
        );
    }
}

#[test]
fn local_nested_generic_method_generated_c_pins_definition_counts() {
    let method_worklist =
        read("tests/integration/generic_specializations/method_worklist_generated_c.rs");

    for required in [
        "generic_method_nested_result.zen",
        r#"assert_c_call_resolves_to_single_definition(&c_source, "Box_wrap_result_i32")"#,
        r#"assert_c_call_resolves_to_single_definition(&c_source, "unwrap_result_Option_i32_StaticString")"#,
        r#"assert_c_call_resolves_to_single_definition(&c_source, "unwrap_option_i32")"#,
    ] {
        assert!(
            method_worklist.contains(required),
            "local nested generic method generated-C tests should pin exact definition counts: {required}"
        );
    }
}

#[test]
fn imported_transitive_worklist_generated_c_pins_definition_counts() {
    let method_worklist =
        read("tests/integration/generic_specializations/multifile_generated_c/method_worklist_dependencies.rs");
    let transitive_block = generated_c_fixture_block(
        &method_worklist,
        "multi_file_generic_imported_transitive_dependency/main.zen",
    );

    for required in [
        r#"assert_c_call_resolves_to_single_definition(&c_source, "inner_i32")"#,
        r#"assert_c_call_resolves_to_single_definition(&c_source, "middle_i32")"#,
        r#"assert_c_call_resolves_to_single_definition(&c_source, "outer_i32")"#,
    ] {
        assert!(
            transitive_block.contains(required),
            "imported transitive generic worklist tests should pin exact definition counts: {required}"
        );
    }
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

#[test]
fn phase5_generic_diagnostics_pin_codes_in_unit_tests() {
    let generic_diagnostics = read("tests/generic_diagnostics.rs");
    let function_inference =
        read("tests/generic_diagnostics/inference_conflicts/functions/basic.rs");
    let method_type_args = read("tests/generic_diagnostics/method_type_args.rs");
    let method_bounds = read("tests/generic_diagnostics/call_site_bounds/methods.rs");
    let generic_bound_validation = read("src/typechecker/generic_bound_validation.rs");

    assert!(
        generic_diagnostics.contains("fn assert_diagnostic_code_and_message("),
        "generic diagnostics tests should have a focused helper for checking code plus message"
    );

    for (source, code) in [
        (function_inference.as_str(), "E5000"),
        (method_type_args.as_str(), "E5001"),
        (method_type_args.as_str(), "E5002"),
        (method_bounds.as_str(), "E6004"),
    ] {
        let normalized = source.split_whitespace().collect::<String>();
        assert!(
            normalized.contains(&format!(
                r#"assert_diagnostic_code_and_message(&errors,"{code}""#
            )),
            "Phase 5 generic diagnostics unit tests should pin diagnostic code {code}"
        );
    }

    assert!(
        !generic_bound_validation.contains("E6012"),
        "generic behavior-bound arity should use public arity code E5001, not stale internal code E6012"
    );
}
