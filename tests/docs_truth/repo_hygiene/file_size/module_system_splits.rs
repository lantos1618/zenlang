use super::super::*;

#[test]
fn module_graph_stdlib_gate_tests_live_in_focused_helper() {
    let root = read("src/module_system/tests/graph_loading.rs");
    let stdlib_gates = read("src/module_system/tests/graph_loading/stdlib_gates.rs");

    for test_name in [
        "module_graph_gates_stdlib_actor_framework_import_before_loading_sketch",
        "module_graph_gates_stdlib_allocator_import_before_loading_sketch",
        "module_graph_gates_stdlib_async_runtime_import_before_loading_sketch",
        "module_graph_gates_stdlib_sync_runtime_import_before_loading_sketch",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "graph_loading.rs should not own graph stdlib gate test: {test_name}"
        );
        assert!(
            stdlib_gates.contains(&format!("fn {test_name}")),
            "module graph stdlib gate tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 210,
        "module graph loading tests should stay focused on graph loading, resolver symbols, visibility, and cycles"
    );
    assert!(
        root.contains("mod stdlib_gates;"),
        "graph_loading.rs should include the focused stdlib_gates module"
    );
}
