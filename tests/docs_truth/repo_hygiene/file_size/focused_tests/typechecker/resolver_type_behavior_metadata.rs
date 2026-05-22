use super::*;

#[test]
fn resolver_type_behavior_type_symbol_tests_stay_split_by_metadata_surface() {
    let root = read("src/typechecker/tests/resolver_type_behavior_metadata/type_symbols.rs");
    let generic_parameters = read(
        "src/typechecker/tests/resolver_type_behavior_metadata/type_symbols/generic_parameters.rs",
    );
    let visibility =
        read("src/typechecker/tests/resolver_type_behavior_metadata/type_symbols/visibility.rs");
    let absent_value_metadata =
        read("src/typechecker/tests/resolver_type_behavior_metadata/type_symbols/absent_value_metadata.rs");

    assert!(
        root.lines().count() < 80,
        "type_symbols.rs should only route focused type-symbol metadata tests"
    );
    for module in [
        "mod absent_value_metadata;",
        "mod generic_parameters;",
        "mod visibility;",
    ] {
        assert!(
            root.contains(module),
            "type_symbols.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "check_program_with_symbols_validates_resolver_type_parameter_counts",
        "check_program_with_symbols_validates_resolver_type_visibility",
        "check_program_with_symbols_validates_resolver_type_like_absent_value_metadata",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "type_symbols.rs should not own concrete test body: {test_name}"
        );
    }
    assert!(
        generic_parameters
            .contains("fn check_program_with_symbols_validates_resolver_type_parameter_counts")
            && generic_parameters
                .contains("fn check_program_with_symbols_validates_resolver_type_parameter_bounds"),
        "generic_parameters.rs should cover type parameter counts, names, and bounds"
    );
    assert!(
        visibility.contains("fn check_program_with_symbols_validates_resolver_type_visibility"),
        "visibility.rs should cover type-symbol visibility metadata"
    );
    assert!(
        absent_value_metadata.contains(
            "fn check_program_with_symbols_validates_resolver_type_like_absent_value_metadata"
        ),
        "absent_value_metadata.rs should cover impossible value metadata on type-like symbols"
    );
}

#[test]
fn resolver_type_behavior_method_tests_stay_split_by_metadata_surface() {
    let root = read("src/typechecker/tests/resolver_type_behavior_metadata/behavior_methods.rs");
    let signature_strings = read(
        "src/typechecker/tests/resolver_type_behavior_metadata/behavior_methods/signature_strings.rs",
    );
    let typed_metadata = read(
        "src/typechecker/tests/resolver_type_behavior_metadata/behavior_methods/typed_metadata.rs",
    );
    let generic_signatures = read(
        "src/typechecker/tests/resolver_type_behavior_metadata/behavior_methods/generic_signatures.rs",
    );

    assert!(
        root.lines().count() < 80,
        "behavior_methods.rs should only route focused behavior method metadata tests"
    );
    for module in [
        "mod generic_signatures;",
        "mod signature_strings;",
        "mod typed_metadata;",
    ] {
        assert!(
            root.contains(module),
            "behavior_methods.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "check_program_with_symbols_validates_resolver_behavior_method_signatures",
        "check_program_with_symbols_validates_resolver_behavior_method_types",
        "check_program_with_symbols_validates_resolver_generic_behavior_method_signatures",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "behavior_methods.rs should not own concrete test body: {test_name}"
        );
    }
    assert!(
        signature_strings
            .contains("fn check_program_with_symbols_validates_resolver_behavior_method_signatures")
            && signature_strings.contains(
                "fn check_program_with_symbols_validates_resolver_behavior_function_type_method_signatures"
            ),
        "signature_strings.rs should cover concrete behavior method signature metadata"
    );
    assert!(
        typed_metadata
            .contains("fn check_program_with_symbols_validates_resolver_behavior_method_types"),
        "typed_metadata.rs should cover typed behavior method metadata"
    );
    assert!(
        generic_signatures.contains(
            "fn check_program_with_symbols_validates_resolver_generic_behavior_method_signatures"
        ) && generic_signatures.contains(
            "fn check_program_with_symbols_validates_resolver_generic_behavior_function_type_method_signatures"
        ),
        "generic_signatures.rs should cover generic behavior method signatures"
    );
}
