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
