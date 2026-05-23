use super::super::*;

#[test]
fn build_graph_core_methods_live_in_focused_module() {
    let root = read("src/build_graph.rs");
    let graph = read("src/build_graph/graph.rs");

    for method in ["pub fn from_input(", "pub fn canonical_json("] {
        assert!(
            !root.contains(method),
            "build_graph.rs should not own BuildGraph core method `{method}`"
        );
        assert!(
            graph.contains(method),
            "BuildGraph core method should live in graph.rs: {method}"
        );
    }

    assert!(
        root.lines().count() < 210,
        "build_graph.rs should stay focused on public build graph data types"
    );
    assert!(
        root.contains("mod graph;"),
        "build_graph.rs should include the focused graph implementation module"
    );
}

#[test]
fn build_graph_target_lowering_tests_stay_split_by_surface() {
    let root = read("tests/build_graph/targets.rs");
    let project = read("tests/build_graph/targets/project.rs");
    let kinds = read("tests/build_graph/targets/kinds.rs");
    let control_flow = read("tests/build_graph/targets/control_flow.rs");
    let metadata = read("tests/build_graph/targets/metadata.rs");

    assert!(
        root.lines().count() < 25,
        "targets.rs should only route focused build graph target test modules"
    );
    for module in [
        "mod control_flow;",
        "mod kinds;",
        "mod metadata;",
        "mod project;",
    ] {
        assert!(
            root.contains(module),
            "targets.rs should include focused module `{module}`"
        );
    }

    assert_tests_live_in_focused_module(
        &root,
        &project,
        "project fixture",
        &["parsed_project_build_zen_lowers_to_executable_and_test_graph"],
    );
    assert_tests_live_in_focused_module(
        &root,
        &kinds,
        "target kind",
        &[
            "build_program_lowering_collects_test_target",
            "build_program_lowering_collects_library_target",
            "build_program_lowering_collects_multiple_executable_targets",
        ],
    );
    assert_tests_live_in_focused_module(
        &root,
        &control_flow,
        "control-flow",
        &[
            "build_program_lowering_collects_static_block_targets",
            "build_program_lowering_rejects_dynamic_target_adds",
        ],
    );
    assert_tests_live_in_focused_module(
        &root,
        &metadata,
        "target metadata",
        &["build_program_lowering_collects_target_dependencies_and_features"],
    );
}

fn assert_tests_live_in_focused_module(
    root: &str,
    focused_module: &str,
    label: &str,
    test_names: &[&str],
) {
    for test_name in test_names {
        let fn_name = format!("fn {test_name}");
        assert!(
            !root.contains(&fn_name),
            "{label} build graph test should move out of the root module: {test_name}"
        );
        assert!(
            focused_module.contains(&fn_name),
            "{label} module should keep build graph test: {test_name}"
        );
    }
}
