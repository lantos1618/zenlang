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
        "const STD: &'static str = \"std\"",
        "const AT_STD: &'static str = \"@std\"",
        "impl FromStr for ModuleRootPrefix",
        "impl fmt::Display for ModuleRootPrefix",
        ".find(|prefix| prefix.as_str() == value)",
        "pub(super) fn parse_module_root_prefix",
    ] {
        assert!(
            root_prefix.contains(required),
            "module root spelling should live in ModuleRootPrefix: {required}"
        );
    }
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
