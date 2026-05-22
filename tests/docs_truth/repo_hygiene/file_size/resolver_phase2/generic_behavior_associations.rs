use super::*;

#[test]
fn resolver_phase2_generic_behavior_association_tests_stay_split_by_surface() {
    let root = read("tests/resolver_phase2/generic_behavior_associations.rs");
    let association_refs =
        read("tests/resolver_phase2/generic_behavior_associations/association_refs.rs");
    let generated_gates =
        read("tests/resolver_phase2/generic_behavior_associations/generated_gates.rs");
    let parent_refs = read("tests/resolver_phase2/generic_behavior_associations/parent_refs.rs");
    let parent_type_args =
        read("tests/resolver_phase2/generic_behavior_associations/parent_type_args.rs");

    assert!(
        root.lines().count() < 60,
        "resolver_phase2 generic_behavior_associations.rs should only route focused modules"
    );
    for module in [
        r#"#[path = "generic_behavior_associations/association_refs.rs"]"#,
        r#"#[path = "generic_behavior_associations/generated_gates.rs"]"#,
        r#"#[path = "generic_behavior_associations/parent_refs.rs"]"#,
        r#"#[path = "generic_behavior_associations/parent_type_args.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "generic behavior association root should include focused module path `{module}`"
        );
    }
    assert!(
        !root.contains("fn resolver_records_behavior_parent_names"),
        "generic behavior association root should not own parent-ref test bodies"
    );
    assert!(
        association_refs.contains("fn resolver_records_behavior_impl_and_requires_names"),
        "association_refs.rs should cover impl/requires behavior refs"
    );
    assert!(
        generated_gates.contains("fn resolver_gates_generated_behavior_derive_association"),
        "generated_gates.rs should cover generated association gates"
    );
    assert!(
        parent_refs.contains("fn resolver_records_generic_behavior_parent_names"),
        "parent_refs.rs should cover concrete and generic behavior parent refs"
    );
    assert!(
        parent_type_args
            .contains("fn resolver_accepts_behavior_parent_type_args_from_child_type_params"),
        "parent_type_args.rs should cover child type parameter propagation"
    );
}
