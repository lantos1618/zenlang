use super::super::*;

#[test]
fn module_graph_stdlib_gate_tests_live_in_focused_helper() {
    let root = read("src/module_system/tests/graph_loading.rs");
    let shared = read("src/module_system/tests.rs");
    let stdlib_gates = read("src/module_system/tests/graph_loading/stdlib_gates.rs");

    assert!(
        !root.contains("stdlib_import_is_gated_before_loading_sketch"),
        "graph_loading.rs should not own individual stdlib gate tests"
    );
    assert!(
        shared.contains("const STDLIB_GATE_CASES: &[StdlibGateCase]"),
        "module-system stdlib gate cases should live in one shared matrix"
    );
    assert!(
        stdlib_gates.contains("fn module_graph_gates_stdlib_imports_before_loading_sketches"),
        "module graph stdlib gate tests should consume the shared case matrix"
    );
    assert!(
        stdlib_gates.contains("StdlibGateLoadPath::Graph"),
        "module graph stdlib gate tests should explicitly cover the graph load path"
    );

    assert!(
        root.lines().count() < 210,
        "module graph loading tests should stay focused on graph loading, resolver symbols, visibility, and cycles"
    );
    assert!(
        root.contains("mod stdlib_gates;"),
        "graph_loading.rs should include the focused stdlib_gates module"
    );
}
