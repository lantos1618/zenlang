use super::*;

#[test]
fn resolver_metadata_restoration_tests_stay_split_by_responsibility() {
    let root = read("src/typechecker/tests/resolver_metadata/metadata_restoration.rs");
    let aggregates =
        read("src/typechecker/tests/resolver_metadata/metadata_restoration/aggregates.rs");
    let behavior_refs =
        read("src/typechecker/tests/resolver_metadata/metadata_restoration/behavior_refs.rs");
    let callables =
        read("src/typechecker/tests/resolver_metadata/metadata_restoration/callables.rs");

    assert!(
        root.lines().count() < 80,
        "metadata_restoration.rs should only route focused restoration tests"
    );
    for module in ["mod aggregates;", "mod behavior_refs;", "mod callables;"] {
        assert!(
            root.contains(module),
            "metadata_restoration.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn resolver_struct_fields_from_metadata_restores_field_names_and_defaults"),
        "aggregate restoration tests should live in aggregates.rs"
    );
    assert!(
        aggregates.contains("fn resolver_enum_variants_from_metadata_uses_owner_scoped_payloads"),
        "aggregates.rs should cover enum and struct metadata restoration"
    );
    assert!(
        behavior_refs
            .contains("fn behavior_impl_refs_from_metadata_restores_type_and_behavior_keys"),
        "behavior_refs.rs should cover behavior ref metadata restoration"
    );
    assert!(
        callables.contains("fn resolver_params_from_metadata_preserves_ast_param_shape"),
        "callables.rs should cover callable metadata restoration"
    );
}
