use super::split_guard::{
    assert_file_contains, assert_file_lacks, assert_file_line_count_below,
    assert_needles_moved_to_focused_file,
};

#[test]
fn core_feature_gate_type_tests_live_in_focused_helper() {
    const ROOT: &str = "src/typechecker/tests/core_semantics/feature_gates.rs";

    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/tests/core_semantics/feature_gates/gated_types.rs",
        &[
            "fn typed_allocator_type_is_rejected_as_gated_not_unknown",
            "fn sync_and_async_typed_allocator_modes_are_rejected_as_gated_not_unknown",
            "fn dynamic_string_type_is_rejected_as_allocator_backed_gate",
            "fn sync_async_effect_modes_are_rejected_as_gated_not_unknown",
            "fn actor_framework_types_are_rejected_as_gated_not_unknown",
            "fn bare_actor_framework_types_are_rejected_as_gated_not_unknown",
        ],
        "feature_gates.rs",
        "gated builtin type focused module",
    );
    assert_file_line_count_below(
        ROOT,
        120,
        "feature_gates.rs should stay focused on syntax and method feature gates",
    );
    assert_file_contains(
        ROOT,
        "mod gated_types;",
        "feature_gates.rs should include the focused gated_types module",
    );
    assert_file_contains(
        "src/typechecker/tests/core_semantics.rs",
        "mod feature_gates;",
        "core_semantics.rs should include feature gate tests",
    );
}

#[test]
fn core_assignment_tests_live_in_focused_helper() {
    const ROOT: &str = "src/typechecker/tests/core_semantics/enum_assignment_and_modules.rs";

    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/tests/core_semantics/enum_assignment_and_modules/assignments.rs",
        &[
            "fn assignment_to_immutable_binding_is_error",
            "fn assignment_to_mutable_closure_parameter_is_allowed",
            "fn assignment_type_mismatch_is_error",
        ],
        "enum_assignment_and_modules.rs",
        "assignment focused module",
    );
    assert_file_line_count_below(
        ROOT,
        200,
        "enum_assignment_and_modules.rs should stay focused on enum, module, field, conversion, and fallthrough checks"
    );
    assert_file_contains(
        ROOT,
        "mod assignments;",
        "enum_assignment_and_modules.rs should include the focused assignments module",
    );
}

#[test]
fn intrinsic_gate_tests_live_in_focused_helpers() {
    const ROOT: &str = "src/typechecker/tests/core_semantics/intrinsic_gates.rs";
    let module_dir = "src/typechecker/tests/core_semantics/intrinsic_gates";

    for (module, test_name) in [
        (
            "async_scheduler",
            "async_scheduler_intrinsics_are_rejected_as_gated_not_unknown",
        ),
        (
            "raw_memory",
            "raw_memory_intrinsics_are_rejected_as_allocator_gates",
        ),
        (
            "byte_memory",
            "byte_memory_intrinsics_are_rejected_as_allocator_gates",
        ),
        (
            "raw_pointer",
            "raw_pointer_intrinsics_are_rejected_as_ownership_gates",
        ),
        ("atomic", "atomic_intrinsics_are_rejected_as_effect_gates"),
        (
            "syscall",
            "syscall_intrinsics_are_rejected_as_host_effect_gates",
        ),
        (
            "type_match",
            "primitive_and_enum_type_match_intrinsics_are_rejected_as_gated_not_unknown",
        ),
    ] {
        let focused_path = format!("{module_dir}/{module}.rs");
        let test_needle = format!("fn {test_name}");
        let module_needle = format!("mod {module};");

        assert_needles_moved_to_focused_file(
            ROOT,
            &focused_path,
            &[test_needle.as_str()],
            "intrinsic_gates.rs",
            "gated intrinsic family focused module",
        );
        assert_file_contains(
            ROOT,
            &module_needle,
            "intrinsic_gates.rs should include focused intrinsic module",
        );
    }

    assert_file_lacks(
        ROOT,
        "#[test]",
        "intrinsic_gates.rs should stay as a router and not define tests directly",
    );
    assert_file_line_count_below(
        ROOT,
        80,
        "intrinsic_gates.rs should stay as a small router for gated intrinsic tests",
    );
}

#[test]
fn core_type_substitution_tests_live_in_focused_helper() {
    const ROOT: &str = "src/typechecker/tests/core_semantics/type_helpers.rs";

    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/tests/core_semantics/type_helpers/substitution.rs",
        &[
            "fn substitute_type_basic",
            "fn substitute_type_covers_all_composite_type_shapes",
            "fn substitute_type_preserves_function_type_arguments_in_nested_generics",
        ],
        "type_helpers.rs",
        "type-substitution focused helper",
    );
    assert_file_line_count_below(
        ROOT,
        150,
        "type_helpers.rs should stay focused on compatibility, resolution, coercion, and inference",
    );
    assert_file_contains(
        ROOT,
        "mod substitution;",
        "type_helpers.rs should include the focused substitution module",
    );
}

#[test]
fn core_match_semantics_tests_live_in_focused_helpers() {
    const ROOT: &str = "src/typechecker/tests/core_semantics/match_semantics.rs";

    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/tests/core_semantics/match_semantics/enum_matches.rs",
        &[
            "fn enum_match_missing_variant_is_error",
            "fn enum_match_duplicate_variant_is_error",
            "fn enum_match_unknown_variant_is_error",
            "fn enum_match_payload_shape_is_checked",
            "fn enum_match_wildcard_after_all_variants_is_redundant",
            "fn enum_match_variant_after_wildcard_is_redundant",
        ],
        "match_semantics.rs",
        "enum match focused helper",
    );
    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/tests/core_semantics/match_semantics/bool_matches.rs",
        &[
            "fn bool_match_missing_arm_is_error_for_value_match",
            "fn bool_match_duplicate_arm_is_error",
        ],
        "match_semantics.rs",
        "bool match focused helper",
    );
    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/tests/core_semantics/match_semantics/result_types.rs",
        &["fn match_arm_return_does_not_force_never_result_type"],
        "match_semantics.rs",
        "match result-type focused helper",
    );
    assert_file_line_count_below(
        ROOT,
        80,
        "match_semantics.rs should stay as a small router for match semantic tests",
    );
    for module in ["enum_matches", "bool_matches", "result_types"] {
        assert_file_contains(
            ROOT,
            &format!("mod {module};"),
            "match_semantics.rs should include focused match semantic module",
        );
    }
}
