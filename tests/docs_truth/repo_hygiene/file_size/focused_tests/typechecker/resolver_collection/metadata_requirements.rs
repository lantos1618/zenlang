use super::*;

#[test]
fn resolver_metadata_requirement_tests_stay_split_by_responsibility() {
    let root = read("src/typechecker/tests/resolver_metadata/metadata_requirements.rs");
    let aggregates =
        read("src/typechecker/tests/resolver_metadata/metadata_requirements/aggregates.rs");
    let callables =
        read("src/typechecker/tests/resolver_metadata/metadata_requirements/callables.rs");
    let lookup = read("src/typechecker/tests/resolver_metadata/metadata_requirements/lookup.rs");

    assert!(
        root.lines().count() < 80,
        "metadata_requirements.rs should only route focused metadata requirement tests"
    );
    for module in ["mod aggregates;", "mod callables;", "mod lookup;"] {
        assert!(
            root.contains(module),
            "metadata_requirements.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn resolver_struct_field_metadata_requires_field_types"),
        "aggregate metadata requirement tests should live in aggregates.rs"
    );
    assert!(
        aggregates.contains("fn resolver_enum_variant_name_metadata_requires_variant_names"),
        "aggregates.rs should cover aggregate metadata requirements"
    );
    assert!(
        callables.contains("fn resolver_callable_signature_metadata_requires_complete_signature"),
        "callables.rs should cover callable and behavior method metadata requirements"
    );
    assert!(
        lookup.contains("fn resolver_behavior_ref_owner_prefers_exact_then_unique_fallbacks"),
        "lookup.rs should cover resolver metadata lookup helpers"
    );
}
