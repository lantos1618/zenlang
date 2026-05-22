use super::*;

#[test]
fn resolver_value_metadata_tests_stay_split_by_metadata_surface() {
    let root = read("src/typechecker/tests/resolver_value_metadata.rs");
    let signature_metadata =
        read("src/typechecker/tests/resolver_value_metadata/signature_metadata.rs");
    let generic_metadata =
        read("src/typechecker/tests/resolver_value_metadata/generic_metadata.rs");

    assert!(
        root.lines().count() < 80,
        "resolver_value_metadata.rs should only route focused value metadata tests"
    );
    for module in [
        "mod absent_declaration_metadata;",
        "mod generic_metadata;",
        "mod signature_metadata;",
    ] {
        assert!(
            root.contains(module),
            "resolver_value_metadata.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn check_program_with_symbols_validates_resolver_function_visibility"),
        "signature metadata tests should live in signature_metadata.rs"
    );
    assert!(
        signature_metadata.contains(
            "fn check_program_with_symbols_validates_resolver_function_typed_signature_metadata",
        ),
        "signature_metadata.rs should cover resolver-backed function signature metadata"
    );
    assert!(
        generic_metadata.contains(
            "fn check_program_with_symbols_validates_resolver_function_type_parameter_bounds",
        ),
        "generic_metadata.rs should cover resolver-backed generic bound metadata"
    );
    assert!(
        generic_metadata.contains(
            "fn check_program_with_symbols_validates_resolver_function_type_parameter_bound_refs",
        ),
        "generic_metadata.rs should cover resolver-backed generic bound-ref metadata"
    );
}
