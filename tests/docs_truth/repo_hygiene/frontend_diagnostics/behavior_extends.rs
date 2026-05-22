use super::*;

#[test]
fn behavior_extends_overlap_diagnostics_stay_split_by_overlap_shape() {
    let root = read("tests/integration/frontend_diagnostics/behavior_extends/overlaps.rs");
    let direct = read("tests/integration/frontend_diagnostics/behavior_extends/overlaps/direct.rs");
    let duplicates =
        read("tests/integration/frontend_diagnostics/behavior_extends/overlaps/duplicates.rs");
    let transitive =
        read("tests/integration/frontend_diagnostics/behavior_extends/overlaps/transitive.rs");

    assert!(
        root.lines().count() < 60,
        "behavior_extends/overlaps.rs should route focused overlap diagnostic modules"
    );
    assert!(
        !root.contains("#[test]"),
        "behavior_extends/overlaps.rs should not own concrete diagnostic tests"
    );
    for module in [
        r#"#[path = "overlaps/direct.rs"]"#,
        r#"#[path = "overlaps/duplicates.rs"]"#,
        r#"#[path = "overlaps/transitive.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "behavior_extends/overlaps.rs should include focused module path `{module}`"
        );
    }
    assert!(
        direct.contains("fn imported_behavior_extends_parent_impl_overlap_is_error"),
        "direct.rs should cover direct imported parent overlap diagnostics"
    );
    assert!(
        transitive.contains("fn imported_behavior_extends_transitive_parent_impl_overlap_is_error"),
        "transitive.rs should cover transitive imported parent overlap diagnostics"
    );
    assert!(
        duplicates.contains("fn imported_duplicate_generic_behavior_impl_is_error"),
        "duplicates.rs should cover duplicate imported generic behavior impl diagnostics"
    );
}
