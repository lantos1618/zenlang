use super::super::*;

#[test]
fn source_dependency_callable_helpers_live_in_focused_module() {
    let root = read("src/typechecker/resolver_validation/imports_source_dependencies.rs");
    let callables =
        read("src/typechecker/resolver_validation/imports_source_dependency_callables.rs");
    let includes = read("src/typechecker/resolver_validation.rs");

    for helper in [
        "fn insert_source_import_type_method_dependencies(",
        "fn insert_source_imported_type_method_dependency(",
        "fn insert_source_function_dependency(",
        "fn insert_source_method_dependency(",
        "fn insert_source_callable_dependency(",
    ] {
        assert!(
            !root.contains(helper),
            "imports_source_dependencies.rs should not own callable helper `{helper}`"
        );
        assert!(
            callables.contains(helper),
            "imported callable source dependency helper should live in focused module: {helper}"
        );
    }

    assert!(
        root.lines().count() < 180,
        "imports_source_dependencies.rs should stay focused on dependency collection and type metadata"
    );
    assert!(
        includes
            .contains("include!(\"resolver_validation/imports_source_dependency_callables.rs\");"),
        "resolver_validation.rs should include focused callable dependency helpers"
    );
}

#[test]
fn imported_declaration_graph_seeding_lives_in_focused_module() {
    let root = read("src/typechecker/resolver_validation/imports_dependencies.rs");
    let graph_seeding = read("src/typechecker/resolver_validation/imports_graph_seeding.rs");
    let includes = read("src/typechecker/resolver_validation.rs");

    for helper in ["fn seed_module_graph_import("] {
        assert!(
            !root.contains(helper),
            "imports_dependencies.rs should not own imported graph seeding helper `{helper}`"
        );
        assert!(
            graph_seeding.contains(helper),
            "imported graph seeding helper should live in focused module: {helper}"
        );
    }

    assert!(
        root.lines().count() < 190,
        "imports_dependencies.rs should stay focused on imported dependency traversal"
    );
    assert!(
        includes.contains("include!(\"resolver_validation/imports_graph_seeding.rs\");"),
        "resolver_validation.rs should include focused imported graph seeding helpers"
    );
}

#[test]
fn replay_behavior_association_tasks_live_in_focused_module() {
    let root = read("src/typechecker/resolver_validation/replay_tasks.rs");
    let associations = read("src/typechecker/resolver_validation/replay_task_association_lists.rs");
    let includes = read("src/typechecker/resolver_validation.rs");

    for helper in [
        "fn collect_resolver_behavior_association_list_tasks",
        "fn collect_resolver_behavior_association_list_tasks_from_declaration_tasks",
        "fn push_resolver_type_behavior_association_list_task",
        "fn push_resolver_behavior_parent_list_task",
    ] {
        assert!(
            !root.contains(helper),
            "replay_tasks.rs should not own behavior association helper `{helper}`"
        );
        assert!(
            associations.contains(helper),
            "behavior association replay helper should live in focused module: {helper}"
        );
    }

    assert!(
        root.lines().count() < 200,
        "replay_tasks.rs should stay focused on declaration replay collection"
    );
    assert!(
        includes.contains("include!(\"resolver_validation/replay_task_association_lists.rs\");"),
        "resolver_validation.rs should include focused behavior association replay helpers"
    );
}

#[test]
fn expected_behavior_edge_helpers_live_in_focused_support_module() {
    let root = read("src/typechecker/resolver_validation_support/behavior_refs.rs");
    let edges = read("src/typechecker/resolver_validation_support/expected_behavior_edges.rs");
    let includes = read("src/typechecker/resolver_validation_support.rs");

    for helper in [
        "struct ExpectedBehaviorEdge",
        "struct ExpectedBehaviorEdgeMetadata",
        "struct ExpectedBehaviorEdges",
        "struct ExpectedBehaviorAssociations",
    ] {
        assert!(
            !root.contains(helper),
            "behavior_refs.rs should not own expected behavior edge helper `{helper}`"
        );
        assert!(
            edges.contains(helper),
            "expected behavior edge helper should live in focused module: {helper}"
        );
    }

    assert!(
        root.lines().count() < 170,
        "behavior_refs.rs should stay focused on role validation and actual metadata selection"
    );
    assert!(
        includes.contains("include!(\"resolver_validation_support/expected_behavior_edges.rs\");"),
        "resolver_validation_support.rs should include focused expected behavior edge helpers"
    );
}

#[test]
fn scalar_absence_descriptors_live_in_focused_support_module() {
    let root = read("src/typechecker/resolver_validation_support/absence_symbol_descriptors.rs");
    let scalars = read("src/typechecker/resolver_validation_support/absence_scalar_descriptors.rs");
    let includes = read("src/typechecker/resolver_validation_support.rs");

    for helper in [
        "struct MutabilityAbsenceValidation",
        "struct SourceAbsenceValidation",
    ] {
        assert!(
            !root.contains(helper),
            "absence_symbol_descriptors.rs should not own scalar absence descriptor `{helper}`"
        );
        assert!(
            scalars.contains(helper),
            "scalar absence descriptor should live in focused module: {helper}"
        );
    }

    assert!(
        root.lines().count() < 190,
        "absence_symbol_descriptors.rs should stay focused on behavior absence descriptors"
    );
    assert!(
        includes
            .contains("include!(\"resolver_validation_support/absence_scalar_descriptors.rs\");"),
        "resolver_validation_support.rs should include focused scalar absence descriptors"
    );
}
