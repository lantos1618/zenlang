use super::*;

#[test]
fn imported_generic_behavior_diagnostics_stay_split_by_responsibility() {
    let root = read("tests/integration/frontend_diagnostics/generic_behavior_imports.rs");

    assert!(
        root.lines().count() < 80,
        "imported generic behavior diagnostics should stay as a small module router"
    );
    for required_module in ["mod arity;", "mod duplicate_requires;", "mod requirements;"] {
        assert!(
            root.contains(required_module),
            "generic behavior import diagnostics should include focused module: {required_module}"
        );
    }

    let arity = read("tests/integration/frontend_diagnostics/generic_behavior_imports/arity.rs");
    let duplicates = read(
        "tests/integration/frontend_diagnostics/generic_behavior_imports/duplicate_requires.rs",
    );
    let requirements =
        read("tests/integration/frontend_diagnostics/generic_behavior_imports/requirements.rs");

    assert!(
        arity.contains("imported_generic_behavior_impl_type_arg_arity_is_error"),
        "arity diagnostics should live in arity.rs"
    );
    assert!(
        duplicates.contains("imported_duplicate_generic_behavior_requires_is_error"),
        "duplicate requires diagnostics should live in duplicate_requires.rs"
    );
    assert!(
        requirements.contains("imported_generic_behavior_requires_missing_impl_is_error"),
        "missing required impl diagnostics should live in requirements.rs"
    );
}

#[test]
fn imported_generic_call_diagnostics_pin_function_method_and_ufc_cases() {
    let root = read("tests/integration/frontend_diagnostics.rs");
    let calls = read("tests/integration/frontend_diagnostics/imported_generic_calls.rs");

    assert!(
        root.contains("mod imported_generic_calls;"),
        "frontend diagnostics root should include focused imported generic call diagnostics"
    );

    for required in [
        "imported_generic_function_inference_conflict_is_error",
        "imported_generic_method_inference_conflict_is_error",
        "imported_generic_ufc_explicit_type_arg_arity_is_error",
        "imported_nongeneric_ufc_explicit_type_args_are_error",
        "imported_generic_ufc_behavior_bound_failure_is_error",
        "conflicting inferred type argument `T` for generic function `choose`",
        "conflicting inferred type argument `T` for generic method `Box.choose`",
        "non-generic function `id_i32` does not accept type arguments",
        "type `Point` does not implement behavior `Json` required by `T`",
        r#""E5000""#,
        r#""E5001""#,
        r#""E5002""#,
        r#""E6004""#,
        "assert_no_diagnostic_message(",
        r#""argument 2""#,
        r#""has no method `encode`""#,
    ] {
        assert!(
            calls.contains(required),
            "imported generic call diagnostics should pin evidence: {required}"
        );
    }
}
