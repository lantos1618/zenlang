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
