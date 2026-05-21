use super::*;

mod graph_loading;

#[test]
fn module_system_roots_use_owned_prefix_enum() {
    let import_resolution = read("src/module_system/import_resolution.rs");
    let graph_loading = read("src/module_system/graph_loading.rs");
    let root_prefix = read("src/module_system/root_prefix.rs");

    for (path, source) in [
        ("src/module_system/import_resolution.rs", import_resolution),
        ("src/module_system/graph_loading.rs", graph_loading),
    ] {
        for forbidden in [
            r#"first == "std""#,
            r#"first == "@std""#,
            r#"first == "@builtin""#,
            r#"match first.as_str()"#,
            r#""std" =>"#,
            r#""@std" =>"#,
            r#""@builtin" =>"#,
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} should parse module roots through ModuleRootPrefix, not raw spelling checks: {forbidden}"
            );
        }
    }

    for required in [
        "enum ModuleRootPrefix",
        "const STD: &'static str = \"std\"",
        "const AT_STD: &'static str = \"@std\"",
        "const AT_BUILTIN: &'static str = \"@builtin\"",
        "impl FromStr for ModuleRootPrefix",
        "impl fmt::Display for ModuleRootPrefix",
        ".find(|prefix| prefix.as_str() == value)",
        "pub(super) fn parse_module_root_prefix",
        "pub(super) const fn is_builtin",
    ] {
        assert!(
            root_prefix.contains(required),
            "module root spelling should live in ModuleRootPrefix: {required}"
        );
    }
}

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

#[test]
fn stdlib_import_gates_live_in_focused_helper() {
    let import_resolution = read("src/module_system/import_resolution.rs");
    let stdlib_gates = read("src/module_system/import_resolution/stdlib_gates.rs");

    assert!(
        !import_resolution.contains("enum GatedStdlibModule"),
        "gated stdlib module table should not live in the import routing dispatcher"
    );

    for required in [
        "pub(super) enum GatedStdlibModule",
        "ActorFramework",
        "AllocatorFramework",
        "AsyncRuntime",
        "SyncRuntime",
        "pub(super) fn from_sub_path",
        "pub(super) fn gate_message",
    ] {
        assert!(
            stdlib_gates.contains(required),
            "stdlib import gate spelling should live in the focused helper: {required}"
        );
    }
}

#[test]
fn stdlib_path_resolution_lives_in_focused_helper() {
    let import_resolution = read("src/module_system/import_resolution.rs");
    let stdlib_paths = read("src/module_system/import_resolution/stdlib_paths.rs");

    for helper in ["resolve_stdlib_file_path", "find_stdlib_root"] {
        assert!(
            !import_resolution.contains(&format!("fn {helper}")),
            "import routing dispatcher should not own stdlib path helper: {helper}"
        );
        assert!(
            stdlib_paths.contains(&format!("fn {helper}")),
            "stdlib path helper should live in focused helper: {helper}"
        );
    }

    assert!(
        import_resolution.contains("mod stdlib_paths;"),
        "import routing should load focused stdlib path helper"
    );
}

#[test]
fn local_import_resolution_lives_in_focused_helper() {
    let import_resolution = read("src/module_system/import_resolution.rs");
    let local_imports = read("src/module_system/import_resolution/local_imports.rs");

    for helper in [
        "local_import_file_path",
        "reject_duplicate_requested_imports",
    ] {
        assert!(
            !import_resolution.contains(&format!("fn {helper}")),
            "import routing dispatcher should not own local import helper: {helper}"
        );
        assert!(
            local_imports.contains(&format!("fn {helper}")),
            "local import helper should live in focused helper: {helper}"
        );
    }

    assert!(
        import_resolution.lines().count() < 190,
        "import routing dispatcher should stay focused on routing imports"
    );
    assert!(
        import_resolution.contains("mod local_imports;"),
        "import routing should load focused local import helper"
    );
}

#[test]
fn imported_declaration_selection_lives_in_focused_helper() {
    let import_resolution = read("src/module_system/import_resolution.rs");
    let imported_declarations =
        read("src/module_system/import_resolution/imported_declarations.rs");

    assert!(
        !import_resolution.contains("fn collect_imported_declarations"),
        "import routing dispatcher should not own imported declaration selection"
    );
    assert!(
        imported_declarations.contains("fn collect_imported_declarations"),
        "imported declaration selection should live in focused helper"
    );
    assert!(
        import_resolution.lines().count() < 240,
        "import routing dispatcher should stay focused on routing imports"
    );
    assert!(
        import_resolution.contains("mod imported_declarations;"),
        "import routing should load focused imported declaration helper"
    );
}
