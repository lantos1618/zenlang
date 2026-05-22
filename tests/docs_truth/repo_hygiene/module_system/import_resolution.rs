use super::*;

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
