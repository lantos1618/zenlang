use super::*;

#[test]
fn typechecker_root_imports_use_static_root_enum() {
    let scope_management = read("src/typechecker/scope_management.rs");
    let import_roots = read("src/typechecker/import_roots.rs");

    for forbidden in [
        r#"["std".to_string()]"#,
        r#"["@std".to_string()]"#,
        r#"path == &["std".to_string()]"#,
        r#"path == &["@std".to_string()]"#,
        r#"path == &["#,
    ] {
        assert!(
            !scope_management.contains(forbidden),
            "typechecker import root checks should not allocate strings for static import roots: {forbidden}"
        );
    }

    for required in [
        "enum RootImportPath",
        "const STD: &'static str = \"std\"",
        "const AT_STD: &'static str = \"@std\"",
        "const ALL: &[RootImportPath]",
        "impl fmt::Display for RootImportPath",
        ".find(|root| root.matches_path(path))",
        "pub(super) fn parse_root_import_path",
    ] {
        assert!(
            import_roots.contains(required),
            "typechecker root import spelling should live in RootImportPath: {required}"
        );
    }
}
