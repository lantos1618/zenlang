use super::super::*;

#[test]
fn core_feature_gate_type_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/core_semantics/feature_gates.rs");
    let gated_types = read("src/typechecker/tests/core_semantics/feature_gates/gated_types.rs");
    let module = read("src/typechecker/tests/core_semantics.rs");

    for test_name in [
        "typed_allocator_type_is_rejected_as_gated_not_unknown",
        "sync_and_async_typed_allocator_modes_are_rejected_as_gated_not_unknown",
        "dynamic_string_type_is_rejected_as_allocator_backed_gate",
        "sync_async_effect_modes_are_rejected_as_gated_not_unknown",
        "actor_framework_types_are_rejected_as_gated_not_unknown",
        "bare_actor_framework_types_are_rejected_as_gated_not_unknown",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "feature_gates.rs should not own gated builtin type test: {test_name}"
        );
        assert!(
            gated_types.contains(&format!("fn {test_name}")),
            "gated builtin type tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 120,
        "feature_gates.rs should stay focused on syntax and method feature gates"
    );
    assert!(
        root.contains("mod gated_types;"),
        "feature_gates.rs should include the focused gated_types module"
    );
    assert!(
        module.contains("mod feature_gates;"),
        "core_semantics.rs should include feature gate tests"
    );
}

#[test]
fn core_assignment_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/core_semantics/enum_assignment_and_modules.rs");
    let assignments =
        read("src/typechecker/tests/core_semantics/enum_assignment_and_modules/assignments.rs");

    for test_name in [
        "assignment_to_immutable_binding_is_error",
        "assignment_to_mutable_closure_parameter_is_allowed",
        "assignment_type_mismatch_is_error",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "enum_assignment_and_modules.rs should not own assignment test: {test_name}"
        );
        assert!(
            assignments.contains(&format!("fn {test_name}")),
            "assignment tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 200,
        "enum_assignment_and_modules.rs should stay focused on enum, module, field, conversion, and fallthrough checks"
    );
    assert!(
        root.contains("mod assignments;"),
        "enum_assignment_and_modules.rs should include the focused assignments module"
    );
}

#[test]
fn intrinsic_gate_tests_live_in_focused_helpers() {
    let root = read("src/typechecker/tests/core_semantics/intrinsic_gates.rs");
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
        let focused = read(format!("{module_dir}/{module}.rs"));
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "intrinsic_gates.rs should not own gated intrinsic family test: {test_name}"
        );
        assert!(
            root.contains(&format!("mod {module};")),
            "intrinsic_gates.rs should include focused module `{module}`"
        );
        assert!(
            focused.contains(&format!("fn {test_name}")),
            "gated intrinsic family test should live in focused module `{module}`: {test_name}"
        );
    }

    assert!(
        !root.contains("#[test]"),
        "intrinsic_gates.rs should stay as a router and not define tests directly"
    );
    assert!(
        root.lines().count() < 80,
        "intrinsic_gates.rs should stay as a small router for gated intrinsic tests"
    );
}
