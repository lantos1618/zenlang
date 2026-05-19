use super::*;

#[test]
fn behavior_default_method_synthesis_lives_in_focused_helper() {
    let support = read("src/typechecker/behavior_impl_support.rs");
    let defaults = read("src/typechecker/behavior_impl_support/default_methods.rs");

    for helper in [
        "behavior_default_methods_for_impl",
        "seed_behavior_default_method_signature",
        "impl_methods_include_behavior_method",
        "behavior_methods_with_inherited_substituted",
        "behavior_parent_type_param_substitutions",
    ] {
        assert!(
            !support.contains(&format!("fn {helper}")),
            "behavior impl support should not own default-method synthesis helper: {helper}"
        );
        assert!(
            defaults.contains(&format!("fn {helper}")),
            "default behavior method synthesis should live in focused helper: {helper}"
        );
    }

    assert!(
        support.contains("mod default_methods;"),
        "behavior impl support should load the focused default-method helper"
    );
}
