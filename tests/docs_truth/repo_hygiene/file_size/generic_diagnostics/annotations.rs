use super::*;

#[test]
fn generic_composite_annotation_tests_stay_split_by_type_shape() {
    let root = read("tests/generic_diagnostics/composite_annotations.rs");
    let nested = read("tests/generic_diagnostics/composite_annotations/nested.rs");
    let function_types = read("tests/generic_diagnostics/composite_annotations/function_types.rs");
    let containers = read("tests/generic_diagnostics/composite_annotations/containers.rs");

    assert!(
        root.lines().count() < 60,
        "composite_annotations.rs should route focused generic annotation modules"
    );
    assert!(
        !root.contains("#[test]"),
        "composite_annotations.rs should not own concrete generic annotation tests"
    );
    for module in [
        r#"#[path = "composite_annotations/nested.rs"]"#,
        r#"#[path = "composite_annotations/function_types.rs"]"#,
        r#"#[path = "composite_annotations/containers.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "composite_annotations.rs should include focused module path `{module}`"
        );
    }

    assert!(
        nested.contains("fn nested_generic_annotation_inner_type_arg_arity_is_error"),
        "nested.rs should cover nested generic annotation diagnostics"
    );
    assert!(
        nested.contains("fn nested_generic_instantiation_inner_type_arg_arity_is_error"),
        "nested.rs should cover nested generic instantiation diagnostics"
    );
    assert!(
        function_types.contains("fn function_type_parameter_annotation_type_arg_arity_is_error"),
        "function_types.rs should cover function parameter annotation diagnostics"
    );
    assert!(
        function_types.contains("fn function_type_return_annotation_without_type_args_is_error"),
        "function_types.rs should cover function return annotation diagnostics"
    );
    assert!(
        containers.contains("fn pointer_type_inner_generic_annotation_arity_is_error"),
        "containers.rs should cover pointer inner generic diagnostics"
    );
    assert!(
        containers.contains("fn array_type_inner_generic_annotation_arity_is_error"),
        "containers.rs should cover array inner generic diagnostics"
    );
}
