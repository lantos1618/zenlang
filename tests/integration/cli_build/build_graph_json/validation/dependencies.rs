#[test]
fn emit_json_build_graph_rejects_dependency_shape_errors() {
    use super::super::super::support::EXECUTABLE_DEPENDENCY_SHAPE_CASES;

    for &(targets, diagnostic) in EXECUTABLE_DEPENDENCY_SHAPE_CASES {
        super::assert_emit_json_build_graph_error_contains(targets, diagnostic);
    }
}
