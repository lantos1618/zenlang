use super::super::*;

#[test]
fn generic_call_site_annotation_tests_stay_split_by_annotation_surface() {
    let root = read("tests/generic_diagnostics/call_site_annotations.rs");
    let calls = read("tests/generic_diagnostics/call_site_annotations/calls.rs");
    let casts = read("tests/generic_diagnostics/call_site_annotations/casts.rs");
    let closures = read("tests/generic_diagnostics/call_site_annotations/closures.rs");

    assert!(
        root.lines().count() < 60,
        "call_site_annotations.rs should route focused generic diagnostic modules"
    );
    assert!(
        !root.contains("#[test]"),
        "call_site_annotations.rs should not own concrete diagnostic tests"
    );
    for module in [
        r#"#[path = "call_site_annotations/calls.rs"]"#,
        r#"#[path = "call_site_annotations/casts.rs"]"#,
        r#"#[path = "call_site_annotations/closures.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "call_site_annotations.rs should include focused module path `{module}`"
        );
    }

    assert!(
        calls.contains("fn generic_function_type_arg_annotation_arity_is_error"),
        "calls.rs should cover generic function call annotation diagnostics"
    );
    assert!(
        calls.contains("fn generic_method_type_arg_annotation_without_type_args_is_error"),
        "calls.rs should cover generic method call annotation diagnostics"
    );
    assert!(
        closures.contains("fn closure_param_annotation_type_arg_arity_is_error"),
        "closures.rs should cover closure annotation diagnostics"
    );
    assert!(
        casts.contains("fn cast_target_annotation_without_type_args_is_error"),
        "casts.rs should cover cast target annotation diagnostics"
    );
}

#[test]
fn generic_bound_diagnostic_tests_stay_split_by_bound_surface() {
    let root = read("tests/generic_diagnostics/bounds.rs");
    let constructors = read("tests/generic_diagnostics/bounds/constructors.rs");
    let annotations = read("tests/generic_diagnostics/bounds/annotations.rs");
    let local_annotations = read("tests/generic_diagnostics/bounds/local_annotations.rs");

    assert!(
        root.lines().count() < 60,
        "bounds.rs should route focused generic bound diagnostic modules"
    );
    assert!(
        !root.contains("#[test]"),
        "bounds.rs should not own concrete generic bound diagnostic tests"
    );
    for module in [
        r#"#[path = "bounds/constructors.rs"]"#,
        r#"#[path = "bounds/annotations.rs"]"#,
        r#"#[path = "bounds/local_annotations.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "bounds.rs should include focused module path `{module}`"
        );
    }

    assert!(
        constructors.contains("fn generic_struct_behavior_bound_failure_is_error"),
        "constructors.rs should cover generic struct constructor bound diagnostics"
    );
    assert!(
        constructors.contains("fn generic_enum_behavior_bound_failure_is_error"),
        "constructors.rs should cover generic enum constructor bound diagnostics"
    );
    assert!(
        annotations.contains("fn generic_struct_annotation_bound_failure_is_error"),
        "annotations.rs should cover generic struct annotation bound diagnostics"
    );
    assert!(
        annotations.contains("fn generic_enum_annotation_bound_failure_is_error"),
        "annotations.rs should cover generic enum annotation bound diagnostics"
    );
    assert!(
        local_annotations.contains("fn generic_struct_local_annotation_bound_failure_is_error"),
        "local_annotations.rs should cover generic struct local annotation bound diagnostics"
    );
    assert!(
        local_annotations.contains("fn generic_enum_local_annotation_bound_failure_is_error"),
        "local_annotations.rs should cover generic enum local annotation bound diagnostics"
    );
}

#[test]
fn generic_constructor_diagnostic_tests_stay_split_by_aggregate_kind() {
    let root = read("tests/generic_diagnostics/constructors.rs");
    let structs = read("tests/generic_diagnostics/constructors/structs.rs");
    let enums = read("tests/generic_diagnostics/constructors/enums.rs");

    assert!(
        root.lines().count() < 60,
        "constructors.rs should route focused generic constructor diagnostic modules"
    );
    assert!(
        !root.contains("#[test]"),
        "constructors.rs should not own concrete generic constructor diagnostic tests"
    );
    for module in [
        r#"#[path = "constructors/structs.rs"]"#,
        r#"#[path = "constructors/enums.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "constructors.rs should include focused module path `{module}`"
        );
    }

    assert!(
        structs.contains("fn generic_struct_type_arg_arity_is_error"),
        "structs.rs should cover generic struct constructor arity diagnostics"
    );
    assert!(
        structs.contains("fn nongeneric_struct_constructor_type_args_are_error"),
        "structs.rs should cover non-generic struct constructor type-arg diagnostics"
    );
    assert!(
        enums.contains("fn generic_enum_type_arg_arity_is_error"),
        "enums.rs should cover generic enum constructor arity diagnostics"
    );
    assert!(
        enums.contains("fn nongeneric_enum_constructor_type_args_are_error"),
        "enums.rs should cover non-generic enum constructor type-arg diagnostics"
    );
}
