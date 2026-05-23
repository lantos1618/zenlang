use super::*;

#[test]
fn module_graph_gates_stdlib_imports_before_loading_sketches() {
    assert_stdlib_gate_cases_are_gated_before_loading_sketch(StdlibGateLoadPath::Graph);
}
