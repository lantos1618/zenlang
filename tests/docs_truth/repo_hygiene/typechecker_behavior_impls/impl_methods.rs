use super::*;

#[test]
fn behavior_impl_method_validation_lives_in_focused_helper() {
    let root = read("src/typechecker/mod.rs");
    let validation = read("src/typechecker/behavior_impl_validation.rs");
    let methods = read("src/typechecker/behavior_impl_method_validation.rs");

    assert!(
        validation.lines().count() < 135,
        "behavior impl validation should stay focused on impl setup and duplicate/overlap checks"
    );

    for helper in [
        "validate_behavior_impl_methods",
        "validate_behavior_impl_declared_methods",
        "validate_behavior_impl_required_method",
        "behavior_impl_actual_method_signature",
    ] {
        assert!(
            !validation.contains(&format!("fn {helper}")),
            "behavior impl validation should not own method validation helper: {helper}"
        );
        assert!(
            methods.contains(&format!("fn {helper}")),
            "behavior impl method validation should live in focused helper: {helper}"
        );
    }

    for diagnostic in [
        "method `{}` is not declared by behavior `{}`",
        "type `{}` implementation of `{}` is missing required method `{}`",
        "method `{}` for behavior `{}` expects {} parameters, found {}",
        "parameter {} for method `{}` in behavior `{}` expects `{}`, found `{}`",
        "method `{}` for behavior `{}` expects return `{}`, found `{}`",
    ] {
        assert!(
            methods.contains(diagnostic),
            "behavior impl method validation should own method diagnostic: {diagnostic}"
        );
    }

    assert!(
        root.contains("mod behavior_impl_method_validation;"),
        "typechecker root should include focused behavior impl method validation"
    );
}
