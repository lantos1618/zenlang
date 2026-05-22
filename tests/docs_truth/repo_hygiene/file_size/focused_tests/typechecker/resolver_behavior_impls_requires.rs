use super::*;

#[test]
fn resolver_behavior_impls_requires_tests_stay_split_by_metadata_surface() {
    let root = read("src/typechecker/tests/resolver_behavior_impls_requires.rs");
    let impl_metadata =
        read("src/typechecker/tests/resolver_behavior_impls_requires/impl_metadata.rs");
    let requires_metadata =
        read("src/typechecker/tests/resolver_behavior_impls_requires/requires_metadata.rs");
    let extra_metadata =
        read("src/typechecker/tests/resolver_behavior_impls_requires/extra_metadata.rs");

    assert!(
        root.lines().count() < 80,
        "resolver_behavior_impls_requires.rs should only route focused behavior relation metadata tests"
    );
    for module in [
        "mod extra_metadata;",
        "mod impl_metadata;",
        "mod requires_metadata;",
    ] {
        assert!(
            root.contains(module),
            "resolver_behavior_impls_requires.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "check_program_with_symbols_validates_resolver_behavior_impl_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_impl_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_impl_refs",
        "check_program_with_symbols_validates_resolver_behavior_required_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_required_names",
        "check_program_with_symbols_validates_resolver_generic_behavior_required_refs",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_behavior_impls_requires.rs should not own concrete test body: {test_name}"
        );
    }
    assert!(
        impl_metadata
            .contains("fn check_program_with_symbols_validates_resolver_behavior_impl_names"),
        "impl_metadata.rs should cover behavior impl name metadata"
    );
    assert!(
        impl_metadata.contains(
            "fn check_program_with_symbols_validates_resolver_generic_behavior_impl_refs",
        ),
        "impl_metadata.rs should cover generic behavior impl refs"
    );
    assert!(
        requires_metadata
            .contains("fn check_program_with_symbols_validates_resolver_behavior_required_names"),
        "requires_metadata.rs should cover behavior requires name metadata"
    );
    assert!(
        requires_metadata.contains(
            "fn check_program_with_symbols_validates_resolver_generic_behavior_required_refs",
        ),
        "requires_metadata.rs should cover generic behavior requires refs"
    );
    assert!(
        extra_metadata
            .contains("fn check_program_with_symbols_rejects_extra_resolver_behavior_impl_refs",),
        "extra_metadata.rs should keep extra impl metadata rejection tests"
    );
}
