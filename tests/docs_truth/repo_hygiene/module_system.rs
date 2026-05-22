use super::*;

mod graph_loading;
mod graph_types;
mod import_resolution;
mod roots;

#[test]
fn module_system_hygiene_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/module_system.rs");
    let graph_types = read("tests/docs_truth/repo_hygiene/module_system/graph_types.rs");
    let import_resolution =
        read("tests/docs_truth/repo_hygiene/module_system/import_resolution.rs");
    let roots = read("tests/docs_truth/repo_hygiene/module_system/roots.rs");

    assert!(
        root.lines().count() < 80,
        "module_system.rs should route focused module-system guard modules"
    );
    for module_name in ["graph_loading", "graph_types", "import_resolution", "roots"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "module_system.rs should include focused module: {module_name}"
        );
    }
    assert!(
        graph_types.contains("fn module_graph_types_live_in_focused_helper"),
        "module graph type guards should live in graph_types.rs"
    );
    assert!(
        import_resolution.contains("fn stdlib_import_gates_live_in_focused_helper")
            && import_resolution
                .contains("fn imported_declaration_selection_lives_in_focused_helper"),
        "import resolution guards should live in import_resolution.rs"
    );
    assert!(
        roots.contains("fn module_system_roots_use_owned_prefix_enum"),
        "module root prefix guards should live in roots.rs"
    );
}
