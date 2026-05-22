use super::*;

#[test]
fn module_graph_types_live_in_focused_helper() {
    let root = read("src/module_system/mod.rs");
    let graph = read("src/module_system/graph.rs");

    for helper in [
        "ModuleId",
        "PackageId",
        "ModuleInfo",
        "ImportBinding",
        "ResolvedModule",
        "ResolvedModuleGraph",
    ] {
        assert!(
            !root.contains(&format!("struct {helper}")),
            "module system root should not own graph type: {helper}"
        );
        assert!(
            graph.contains(&format!("struct {helper}")),
            "module graph type should live in graph.rs: {helper}"
        );
    }

    assert!(
        root.contains("mod graph;")
            && root.contains("pub use graph::{")
            && root.contains("ResolvedModuleGraph"),
        "module system root should load and re-export focused graph types"
    );
    assert!(
        root.lines().count() < 180,
        "module system root should stay focused on loading, caching, and IDs"
    );
}
