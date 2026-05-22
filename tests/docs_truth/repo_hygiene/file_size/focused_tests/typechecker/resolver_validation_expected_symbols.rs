use super::*;

#[test]
fn resolver_validation_expected_symbol_tests_stay_split_by_symbol_surface() {
    let root = read("src/typechecker/tests/resolver_validation/expected_symbols.rs");
    let composite =
        read("src/typechecker/tests/resolver_validation/expected_symbols/composite_symbols.rs");
    let leaf = read("src/typechecker/tests/resolver_validation/expected_symbols/leaf_symbols.rs");
    let signature_parts =
        read("src/typechecker/tests/resolver_validation/expected_symbols/signature_parts.rs");
    let value_type_symbols =
        read("src/typechecker/tests/resolver_validation/expected_symbols/value_type_symbols.rs");

    assert!(
        root.lines().count() < 80,
        "expected_symbols.rs should only route focused expected-symbol test modules"
    );
    for module in [
        "mod composite_symbols;",
        "mod leaf_symbols;",
        "mod signature_parts;",
        "mod value_type_symbols;",
    ] {
        assert!(
            root.contains(module),
            "expected_symbols.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn expected_parameter_builds_name_display_and_type_together")
            && !root.contains("fn expected_value_symbol_builds_signature_and_visibility_together"),
        "expected_symbols.rs should not own concrete expected-symbol helper tests"
    );

    assert!(
        signature_parts.contains("fn expected_parameter_builds_name_display_and_type_together")
            && signature_parts
                .contains("fn expected_return_metadata_defaults_and_displays_together")
            && signature_parts
                .contains("fn expected_type_parameter_builds_bound_display_and_ref_together")
            && signature_parts.contains("fn expected_field_builds_display_and_type_together")
            && signature_parts
                .contains("fn expected_variant_payload_builds_display_and_type_together"),
        "signature_parts.rs should cover expected metadata component helpers"
    );
    assert!(
        value_type_symbols
            .contains("fn expected_behavior_method_builds_signature_and_metadata_together")
            && value_type_symbols
                .contains("fn expected_value_signature_builds_components_together")
            && value_type_symbols
                .contains("fn expected_value_symbol_builds_signature_and_visibility_together")
            && value_type_symbols.contains(
                "fn expected_type_like_symbol_builds_type_params_and_visibility_together"
            ),
        "value_type_symbols.rs should cover expected value/type-like symbol helpers"
    );
    assert!(
        composite.contains("fn expected_struct_symbol_builds_type_like_and_fields_together")
            && leaf.contains("fn expected_import_symbol_builds_source_and_visibility_together"),
        "existing composite and leaf expected-symbol modules should remain intact"
    );
}
