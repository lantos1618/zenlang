use super::split_guard::{
    assert_file_contains, assert_file_line_count_below, assert_needles_moved_to_focused_file,
};

#[test]
fn source_dependency_callable_helpers_live_in_focused_module() {
    const ROOT: &str = "src/typechecker/resolver_validation/imports_source_dependencies.rs";

    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/resolver_validation/imports_source_dependency_callables.rs",
        &[
            "fn insert_source_import_type_method_dependencies(",
            "fn insert_source_imported_type_method_dependency(",
            "fn insert_source_function_dependency(",
            "fn insert_source_method_dependency(",
            "fn insert_source_callable_dependency(",
        ],
        "imports_source_dependencies.rs",
        "imported callable source dependency focused module",
    );
    assert_file_line_count_below(
        ROOT,
        180,
        "imports_source_dependencies.rs should stay focused on dependency collection and type metadata"
    );
    assert_file_contains(
        "src/typechecker/resolver_validation.rs",
        "include!(\"resolver_validation/imports_source_dependency_callables.rs\");",
        "resolver_validation.rs should include focused callable dependency helpers",
    );
}

#[test]
fn imported_declaration_graph_seeding_lives_in_focused_module() {
    const ROOT: &str = "src/typechecker/resolver_validation/imports_dependencies.rs";

    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/resolver_validation/imports_graph_seeding.rs",
        &["fn seed_module_graph_import("],
        "imports_dependencies.rs",
        "imported graph seeding focused module",
    );
    assert_file_line_count_below(
        ROOT,
        190,
        "imports_dependencies.rs should stay focused on imported dependency traversal",
    );
    assert_file_contains(
        "src/typechecker/resolver_validation.rs",
        "include!(\"resolver_validation/imports_graph_seeding.rs\");",
        "resolver_validation.rs should include focused imported graph seeding helpers",
    );
}

#[test]
fn imported_behavior_extends_helpers_live_in_focused_module() {
    const ROOT: &str = "src/typechecker/resolver_validation/imports_behavior_dependencies.rs";

    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/resolver_validation/imports_behavior_extends.rs",
        &[
            "fn seed_behavior_extends_for_imported_behavior(",
            "fn seed_behavior_extends_for_imported_behavior_inner(",
        ],
        "imports_behavior_dependencies.rs",
        "imported behavior-extends focused module",
    );
    assert_file_line_count_below(
        ROOT,
        190,
        "imports_behavior_dependencies.rs should stay focused on impl dependency seeding",
    );
    assert_file_contains(
        "src/typechecker/resolver_validation.rs",
        "include!(\"resolver_validation/imports_behavior_extends.rs\");",
        "resolver_validation.rs should include focused imported behavior-extends helpers",
    );
}

#[test]
fn replay_behavior_association_tasks_live_in_focused_module() {
    const ROOT: &str = "src/typechecker/resolver_validation/replay_tasks.rs";

    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/resolver_validation/replay_task_association_lists.rs",
        &[
            "fn collect_resolver_behavior_association_list_tasks",
            "fn collect_resolver_behavior_association_list_tasks_from_declaration_tasks",
            "fn push_resolver_type_behavior_association_list_task",
            "fn push_resolver_behavior_parent_list_task",
        ],
        "replay_tasks.rs",
        "behavior association replay focused module",
    );
    assert_file_line_count_below(
        ROOT,
        200,
        "replay_tasks.rs should stay focused on declaration replay collection",
    );
    assert_file_contains(
        "src/typechecker/resolver_validation.rs",
        "include!(\"resolver_validation/replay_task_association_lists.rs\");",
        "resolver_validation.rs should include focused behavior association replay helpers",
    );
}

#[test]
fn expected_behavior_edge_helpers_live_in_focused_support_module() {
    const ROOT: &str = "src/typechecker/resolver_validation_support/behavior_refs.rs";

    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/resolver_validation_support/expected_behavior_edges.rs",
        &[
            "struct ExpectedBehaviorEdge",
            "struct ExpectedBehaviorEdgeMetadata",
            "struct ExpectedBehaviorEdges",
            "struct ExpectedBehaviorAssociations",
        ],
        "behavior_refs.rs",
        "expected behavior edge focused support module",
    );
    assert_file_line_count_below(
        ROOT,
        170,
        "behavior_refs.rs should stay focused on role validation and actual metadata selection",
    );
    assert_file_contains(
        "src/typechecker/resolver_validation_support.rs",
        "include!(\"resolver_validation_support/expected_behavior_edges.rs\");",
        "resolver_validation_support.rs should include focused expected behavior edge helpers",
    );
}

#[test]
fn scalar_absence_descriptors_live_in_focused_support_module() {
    const ROOT: &str = "src/typechecker/resolver_validation_support/absence_symbol_descriptors.rs";

    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/resolver_validation_support/absence_scalar_descriptors.rs",
        &[
            "struct MutabilityAbsenceValidation",
            "struct SourceAbsenceValidation",
        ],
        "absence_symbol_descriptors.rs",
        "scalar absence descriptor focused support module",
    );
    assert_file_line_count_below(
        ROOT,
        190,
        "absence_symbol_descriptors.rs should stay focused on behavior absence descriptors",
    );
    assert_file_contains(
        "src/typechecker/resolver_validation_support.rs",
        "include!(\"resolver_validation_support/absence_scalar_descriptors.rs\");",
        "resolver_validation_support.rs should include focused scalar absence descriptors",
    );
}

#[test]
fn expected_pattern_local_traversal_lives_in_focused_support_module() {
    const ROOT: &str = "src/typechecker/resolver_validation_support/expected_local_traversal.rs";

    assert_needles_moved_to_focused_file(
        ROOT,
        "src/typechecker/resolver_validation_support/expected_pattern_locals.rs",
        &[
            "fn expected_resolver_pattern_locals(",
            "fn expected_resolver_pattern_binding(",
        ],
        "expected_local_traversal.rs",
        "expected pattern-local focused support module",
    );
    assert_file_line_count_below(
        ROOT,
        200,
        "expected_local_traversal.rs should stay focused on expression and statement traversal",
    );
    assert_file_contains(
        "src/typechecker/resolver_validation_support.rs",
        "include!(\"resolver_validation_support/expected_pattern_locals.rs\");",
        "resolver_validation_support.rs should include focused expected pattern-local helpers",
    );
}
