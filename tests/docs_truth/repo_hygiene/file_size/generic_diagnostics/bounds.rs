use super::*;

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
