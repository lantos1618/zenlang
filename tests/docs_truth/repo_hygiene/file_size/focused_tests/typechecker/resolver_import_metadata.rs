use super::*;

#[test]
fn resolver_import_module_metadata_tests_stay_split_by_metadata_surface() {
    let root = read("src/typechecker/tests/resolver_import_metadata/module_metadata.rs");
    let source_visibility =
        read("src/typechecker/tests/resolver_import_metadata/module_metadata/source_visibility.rs");
    let absent_declaration = read(
        "src/typechecker/tests/resolver_import_metadata/module_metadata/absent_declaration.rs",
    );
    let absent_type =
        read("src/typechecker/tests/resolver_import_metadata/module_metadata/absent_type.rs");

    assert!(
        root.lines().count() < 80,
        "module_metadata.rs should only route focused module metadata tests"
    );
    for module in [
        "mod absent_declaration;",
        "mod absent_type;",
        "mod source_visibility;",
    ] {
        assert!(
            root.contains(module),
            "module_metadata.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "check_program_with_symbols_validates_resolver_module_symbols",
        "check_program_with_symbols_validates_resolver_module_absent_declaration_metadata",
        "check_program_with_symbols_validates_resolver_module_absent_type_metadata",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "module_metadata.rs should not own concrete test body: {test_name}"
        );
    }
    assert!(
        source_visibility
            .contains("fn check_program_with_symbols_validates_resolver_module_symbols"),
        "source_visibility.rs should cover module source and visibility metadata"
    );
    assert!(
        absent_declaration.contains(
            "fn check_program_with_symbols_validates_resolver_module_absent_declaration_metadata",
        ),
        "absent_declaration.rs should cover forbidden declaration metadata on module symbols"
    );
    assert!(
        absent_type.contains(
            "fn check_program_with_symbols_validates_resolver_module_absent_type_metadata",
        ),
        "absent_type.rs should cover forbidden type metadata on module symbols"
    );
}
