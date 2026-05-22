use super::*;

#[test]
fn resolver_phase2_value_metadata_tests_live_in_focused_modules() {
    let root = read("tests/resolver_phase2/value_metadata.rs");
    let generic_metadata = read("tests/resolver_phase2/value_metadata/generic_metadata.rs");
    let signature_metadata = read("tests/resolver_phase2/value_metadata/signature_metadata.rs");

    assert!(
        root.lines().count() < 60,
        "resolver_phase2 value_metadata.rs should only route focused value metadata modules"
    );
    for module in [
        r#"#[path = "value_metadata/generic_metadata.rs"]"#,
        r#"#[path = "value_metadata/signature_metadata.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "resolver_phase2 value_metadata.rs should include focused module path `{module}`"
        );
    }

    for test_name in [
        "resolver_records_value_symbol_parameter_counts",
        "resolver_records_value_symbol_parameter_types",
        "resolver_records_value_symbol_parameter_names",
        "resolver_records_value_symbol_return_types",
        "resolver_records_value_symbol_function_type_metadata",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "signature metadata resolver test should move out of value_metadata.rs: {test_name}"
        );
        assert!(
            signature_metadata.contains(&format!("fn {test_name}")),
            "signature_metadata.rs should keep value metadata resolver test: {test_name}"
        );
    }

    for test_name in [
        "resolver_records_value_symbol_generic_parameter_counts",
        "resolver_records_value_symbol_generic_bounds",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "generic metadata resolver test should move out of value_metadata.rs: {test_name}"
        );
        assert!(
            generic_metadata.contains(&format!("fn {test_name}")),
            "generic_metadata.rs should keep value metadata resolver test: {test_name}"
        );
    }
}
