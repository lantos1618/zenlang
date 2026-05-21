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
fn frontend_diagnostics_root_stays_a_small_router() {
    let root = read("tests/integration/frontend_diagnostics.rs");
    let helper_errors = read("tests/integration/frontend_diagnostics/helper_errors.rs");
    let imported_generic_arity =
        read("tests/integration/frontend_diagnostics/imported_generic_arity.rs");

    assert!(
        root.lines().count() < 80,
        "frontend diagnostics root should only route focused diagnostic modules"
    );
    for required_module in [
        "mod behavior_extends;",
        "mod generic_behavior_imports;",
        "mod helper_errors;",
        "mod imported_generic_arity;",
    ] {
        assert!(
            root.contains(required_module),
            "frontend diagnostics root should include focused module: {required_module}"
        );
    }
    assert!(
        helper_errors.contains("fn integration_frontend_helper_runs_resolver_diagnostics"),
        "helper error tests should live in helper_errors.rs"
    );
    assert!(
        imported_generic_arity
            .contains("fn imported_generic_function_explicit_type_arg_arity_is_error"),
        "imported generic arity tests should live in imported_generic_arity.rs"
    );
    assert!(
        !root.contains("fn imported_generic_function_explicit_type_arg_arity_is_error"),
        "frontend diagnostics root should not contain imported generic arity bodies"
    );
}
