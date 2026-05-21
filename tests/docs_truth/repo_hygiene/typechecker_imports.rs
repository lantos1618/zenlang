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

#[test]
fn typechecker_imported_method_seeding_lives_in_focused_helper() {
    let dependencies = read("src/typechecker/resolver_validation/imports_source_dependencies.rs");
    let seeding = read("src/typechecker/resolver_validation/imported_method_seeding.rs");

    for helper in [
        "seed_imported_method_with_dependencies",
        "seed_imported_impl_method",
        "seed_imported_method_signature",
    ] {
        assert!(
            !dependencies.contains(&format!("fn {helper}")),
            "source dependency collection should not own imported method seeding helper: {helper}"
        );
        assert!(
            seeding.contains(&format!("fn {helper}")),
            "imported method seeding should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/resolver_validation.rs");
    assert!(
        root.contains("include!(\"resolver_validation/imported_method_seeding.rs\");"),
        "resolver validation should include focused imported method seeding"
    );
}

#[test]
fn typechecker_imported_type_method_dependencies_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let dependencies = read("src/typechecker/resolver_validation/imports_source_dependencies.rs");
    let type_methods = read("src/typechecker/resolver_validation/imports_source_type_methods.rs");

    for helper in [
        "insert_source_import_type_method_dependencies",
        "insert_source_imported_type_method_dependency",
    ] {
        assert!(
            !dependencies.contains(&format!("fn {helper}")),
            "source dependency collection should not own imported type-method dependency helper: {helper}"
        );
        assert!(
            type_methods.contains(&format!("fn {helper}")),
            "imported type-method dependency helper should live in focused helper: {helper}"
        );
    }

    assert!(
        dependencies.lines().count() < 205,
        "source dependency collection should stay focused on local source dependency collection"
    );
    assert!(
        root.contains("include!(\"resolver_validation/imports_source_type_methods.rs\");"),
        "resolver validation should include focused imported type-method dependencies"
    );
}
