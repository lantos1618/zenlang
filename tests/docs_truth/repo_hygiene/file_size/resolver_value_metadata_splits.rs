use super::super::*;

#[test]
fn resolver_value_function_metadata_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/resolver_value_metadata.rs");
    let function_metadata =
        read("src/typechecker/tests/resolver_value_metadata/function_metadata.rs");
    let generic_parameters = read(
        "src/typechecker/tests/resolver_value_metadata/function_metadata/generic_parameters.rs",
    );

    for test_name in [
        "check_program_with_symbols_validates_resolver_function_visibility",
        "check_program_with_symbols_validates_resolver_function_return_type",
        "check_program_with_symbols_validates_resolver_function_type_return_metadata",
        "check_program_with_symbols_validates_resolver_function_typed_signature_metadata",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_value_metadata.rs should not own function metadata test: {test_name}"
        );
        assert!(
            function_metadata.contains(&format!("fn {test_name}")),
            "resolver value function metadata tests should live in focused module: {test_name}"
        );
    }

    for test_name in [
        "check_program_with_symbols_validates_resolver_function_type_parameter_counts",
        "check_program_with_symbols_validates_resolver_function_type_parameter_names",
        "check_program_with_symbols_validates_resolver_function_type_parameter_bounds",
        "check_program_with_symbols_validates_resolver_function_type_parameter_bound_refs",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_value_metadata.rs should not own function metadata test: {test_name}"
        );
        assert!(
            !function_metadata.contains(&format!("fn {test_name}")),
            "function_metadata.rs should not own generic parameter test: {test_name}"
        );
        assert!(
            generic_parameters.contains(&format!("fn {test_name}")),
            "resolver value generic parameter metadata tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 80,
        "resolver_value_metadata.rs should stay focused on grouping value metadata replay tests"
    );
    assert!(
        function_metadata.lines().count() < 140,
        "function_metadata.rs should stay focused on function signature metadata replay tests"
    );
    assert!(
        root.contains("mod function_metadata;"),
        "resolver_value_metadata.rs should include the focused function_metadata module"
    );
    assert!(
        function_metadata.contains("mod generic_parameters;"),
        "function_metadata.rs should include the focused generic_parameters module"
    );
}
