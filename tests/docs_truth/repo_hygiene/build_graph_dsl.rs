use super::*;

mod dsl_parsing;
mod graph_root;
mod lowering;
mod target_helpers;

#[test]
fn build_graph_dsl_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/build_graph_dsl.rs");
    let dsl_parsing = read("tests/docs_truth/repo_hygiene/build_graph_dsl/dsl_parsing.rs");
    let graph_root = read("tests/docs_truth/repo_hygiene/build_graph_dsl/graph_root.rs");
    let lowering = read("tests/docs_truth/repo_hygiene/build_graph_dsl/lowering.rs");

    assert!(
        root.lines().count() < 80,
        "build_graph_dsl.rs should route focused build graph hygiene modules"
    );
    for module_name in ["dsl_parsing", "graph_root", "lowering", "target_helpers"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "build_graph_dsl.rs should include focused guard module: {module_name}"
        );
    }
    assert!(
        dsl_parsing.contains("fn build_graph_dsl_parsing_uses_enum_static_tables"),
        "DSL parsing guard should live in build_graph_dsl/dsl_parsing.rs"
    );
    assert!(
        graph_root.contains("fn build_graph_dependency_order_lives_in_focused_helper"),
        "build graph root helper guards should live in build_graph_dsl/graph_root.rs"
    );
    assert!(
        lowering.contains("fn build_graph_ast_traversal_lives_in_focused_helper"),
        "lowering/traversal guards should live in build_graph_dsl/lowering.rs"
    );
}
