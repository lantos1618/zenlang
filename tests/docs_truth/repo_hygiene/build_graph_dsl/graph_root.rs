use super::*;

#[test]
fn build_graph_dependency_order_lives_in_focused_helper() {
    let root = read("src/build_graph.rs");
    let dependency_order = read("src/build_graph/dependency_order.rs");

    for helper in ["targets_in_dependency_order", "visit_target"] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "build graph root should not own dependency ordering helper: {helper}"
        );
        assert!(
            dependency_order.contains(&format!("fn {helper}")),
            "dependency ordering should live in focused build graph helper: {helper}"
        );
    }

    assert!(
        root.contains("mod dependency_order;"),
        "build graph root should include focused dependency ordering module"
    );
}

#[test]
fn build_graph_error_types_live_in_focused_helper() {
    let root = read("src/build_graph.rs");
    let errors = read("src/build_graph/errors.rs");

    for helper in [
        "enum BuildGraphError",
        "impl fmt::Display for BuildGraphError",
        "impl std::error::Error for BuildGraphError",
    ] {
        assert!(
            !root.contains(helper),
            "build graph root should not own error type/formatting helper: {helper}"
        );
        assert!(
            errors.contains(helper),
            "build graph error helper should live in errors.rs: {helper}"
        );
    }

    assert!(
        root.lines().count() < 220,
        "build_graph.rs should stay focused on graph shapes and construction"
    );
    assert!(
        root.contains("mod errors;") && root.contains("pub use errors::BuildGraphError;"),
        "build graph root should include and re-export the focused error helper"
    );
}

#[test]
fn build_graph_json_output_lives_in_focused_helper() {
    let root = read("src/build_graph.rs");
    let json = read("src/build_graph/json.rs");

    for helper in ["struct BuildGraphJson", "fn canonical_json"] {
        assert!(
            !root.contains(helper),
            "build graph root should not own JSON output helper: {helper}"
        );
        assert!(
            json.contains(helper),
            "build graph JSON output should live in focused helper: {helper}"
        );
    }
    assert!(
        root.lines().count() < 200,
        "build_graph.rs should stay focused on graph shapes and construction"
    );
    assert!(
        root.contains("mod json;"),
        "build graph root should include focused JSON output helper"
    );
}

#[test]
fn build_graph_formatting_lives_in_focused_helper() {
    let root = read("src/build_graph.rs");
    let formatting = read("src/build_graph/formatting.rs");

    for helper in [
        "fn diagnostic_name",
        "impl fmt::Display for BuildTargetKind",
        "impl fmt::Display for HostEffect",
    ] {
        assert!(
            !root.contains(helper),
            "build_graph.rs should not own formatting helper: {helper}"
        );
        assert!(
            formatting.contains(helper),
            "build graph formatting should live in focused helper: {helper}"
        );
    }
    assert!(
        root.contains("mod formatting;"),
        "build graph root should include focused formatting helper"
    );
    assert!(
        root.lines().count() < 180,
        "build_graph.rs should stay focused on graph shapes and construction"
    );
}
