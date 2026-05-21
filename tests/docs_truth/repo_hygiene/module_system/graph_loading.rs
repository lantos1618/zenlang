use super::*;

#[test]
fn graph_loading_export_lookup_lives_in_focused_helper() {
    let graph_loading = read("src/module_system/graph_loading.rs");
    let exported_symbols = read("src/module_system/graph_loading/exported_symbols.rs");
    let import_bindings = read("src/module_system/graph_loading/import_bindings.rs");

    assert!(
        graph_loading.lines().count() < 240,
        "module graph loading should stay focused on loading and import traversal"
    );
    for helper in ["enum ExportedModuleSymbol", "fn exported_module_symbol"] {
        assert!(
            !graph_loading.contains(helper),
            "exported module symbol definitions should live in exported_symbols.rs: {helper}"
        );
        assert!(
            exported_symbols.contains(helper),
            "exported_symbols.rs should own exported module symbol definitions: {helper}"
        );
    }
    assert!(
        graph_loading.contains("mod exported_symbols;"),
        "graph loading should include the focused exported-symbol helper"
    );
    assert!(
        import_bindings.contains(
            "use super::exported_symbols::{exported_module_symbol, ExportedModuleSymbol};"
        ),
        "graph import binding collection should import exported-symbol helpers explicitly"
    );
}

#[test]
fn graph_loading_import_bindings_live_in_focused_helper() {
    let graph_loading = read("src/module_system/graph_loading.rs");
    let import_bindings = read("src/module_system/graph_loading/import_bindings.rs");

    assert!(
        graph_loading.lines().count() < 180,
        "module graph loading should stay focused on loading and import traversal"
    );
    for helper in ["fn collect_import_bindings", "exported_module_symbol"] {
        assert!(
            !graph_loading.contains(helper),
            "graph loading root should not own import binding helper: {helper}"
        );
        assert!(
            import_bindings.contains(helper),
            "import_bindings.rs should own graph import binding collection: {helper}"
        );
    }
    assert!(
        graph_loading.contains("mod import_bindings;"),
        "graph loading should include the focused import-binding helper"
    );
}

#[test]
fn module_graph_stdlib_gate_sketch_tests_live_in_focused_helper() {
    let graph_loading = read("src/module_system/tests/graph_loading.rs");
    let stdlib_gates = read("src/module_system/tests/graph_loading/stdlib_gates.rs");

    for helper in [
        "assert_graph_stdlib_import_is_gated_before_loading_sketch",
        "module_graph_gates_stdlib_actor_framework_import_before_loading_sketch",
        "module_graph_gates_stdlib_allocator_import_before_loading_sketch",
        "module_graph_gates_stdlib_async_runtime_import_before_loading_sketch",
        "module_graph_gates_stdlib_sync_runtime_import_before_loading_sketch",
    ] {
        assert!(
            !graph_loading.contains(&format!("fn {helper}")),
            "module graph loading tests should not own stdlib gate sketch helper: {helper}"
        );
        assert!(
            stdlib_gates.contains(&format!("fn {helper}")),
            "stdlib gate sketch test should live in focused helper: {helper}"
        );
    }

    assert!(
        graph_loading.lines().count() < 200,
        "graph_loading.rs tests should stay focused on graph loading behavior"
    );
    assert!(
        graph_loading.contains("mod stdlib_gates;"),
        "graph_loading.rs should include focused stdlib gate sketch tests"
    );
}
