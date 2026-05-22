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
