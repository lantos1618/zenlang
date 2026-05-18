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
