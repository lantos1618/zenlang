use super::*;

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
            r#"match first.as_str()"#,
            r#""std" =>"#,
            r#""@std" =>"#,
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} should parse module roots through ModuleRootPrefix, not raw spelling checks: {forbidden}"
            );
        }
    }

    for required in [
        "enum ModuleRootPrefix",
        "use crate::root_spelling::{AT_STD_ROOT, STD_ROOT};",
        "Self::Std => STD_ROOT",
        "Self::AtStd => AT_STD_ROOT",
        "pub(super) fn parse_module_root_prefix",
    ] {
        assert!(
            root_prefix.contains(required),
            "module root spelling should live in ModuleRootPrefix: {required}"
        );
    }
    assert!(
        owns_static_spelling_from_str(&root_prefix, "ModuleRootPrefix")
            && owns_static_spelling_display(&root_prefix, "ModuleRootPrefix")
            && uses_static_spelling_parser(&root_prefix),
        "module root spelling should use shared static spelling parse/display glue"
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
        "IoUringRuntime",
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

#[test]
fn local_import_resolution_helpers_live_in_focused_helper() {
    let import_resolution = read("src/module_system/import_resolution.rs");
    let local_imports = read("src/module_system/import_resolution/local_imports.rs");

    for helper in [
        "fn local_import_file_path",
        "fn reject_duplicate_requested_imports",
    ] {
        assert!(
            !import_resolution.contains(helper),
            "import routing dispatcher should not own local import helper: {helper}"
        );
        assert!(
            local_imports.contains(helper),
            "local import helper should live in focused helper: {helper}"
        );
    }

    assert!(
        import_resolution.lines().count() < 210,
        "import routing dispatcher should stay focused on routing imports"
    );
    assert!(
        import_resolution.contains("mod local_imports;"),
        "import routing should load focused local import helper"
    );
}

#[test]
fn graph_loading_export_lookup_lives_in_focused_helper() {
    let graph_loading = read("src/module_system/graph_loading.rs");
    let exported_symbols = read("src/module_system/graph_loading/exported_symbols.rs");

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
        graph_loading
            .contains("use exported_symbols::{exported_module_symbol, ExportedModuleSymbol};"),
        "graph loading should import exported-symbol helpers explicitly"
    );
}
