use super::*;

#[test]
fn typechecker_resolver_type_behavior_metadata_tests_live_in_focused_modules() {
    let root = read("src/typechecker/tests/resolver_type_behavior_metadata.rs");
    let type_metadata =
        read("src/typechecker/tests/resolver_type_behavior_metadata/type_symbols.rs");
    let type_generic_parameters = read(
        "src/typechecker/tests/resolver_type_behavior_metadata/type_symbols/generic_parameters.rs",
    );
    let type_visibility =
        read("src/typechecker/tests/resolver_type_behavior_metadata/type_symbols/visibility.rs");
    let type_absent_value_metadata = read(
        "src/typechecker/tests/resolver_type_behavior_metadata/type_symbols/absent_value_metadata.rs",
    );
    let behavior_metadata =
        read("src/typechecker/tests/resolver_type_behavior_metadata/behavior_symbols.rs");

    for test_name in [
        "check_program_with_symbols_validates_resolver_type_parameter_counts",
        "check_program_with_symbols_validates_resolver_type_parameter_names",
        "check_program_with_symbols_validates_resolver_type_parameter_bounds",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_type_behavior_metadata.rs should not own type metadata test: {test_name}"
        );
        assert!(
            !type_metadata.contains(&format!("fn {test_name}")),
            "type_symbols.rs should not own generic parameter test body: {test_name}"
        );
        assert!(
            type_generic_parameters.contains(&format!("fn {test_name}")),
            "generic type-parameter metadata tests should live in focused module: {test_name}"
        );
    }
    for (test_name, module, label) in [
        (
            "check_program_with_symbols_validates_resolver_type_visibility",
            type_visibility.as_str(),
            "type visibility",
        ),
        (
            "check_program_with_symbols_validates_resolver_type_like_absent_value_metadata",
            type_absent_value_metadata.as_str(),
            "type-like absent value metadata",
        ),
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_type_behavior_metadata.rs should not own {label} test: {test_name}"
        );
        assert!(
            !type_metadata.contains(&format!("fn {test_name}")),
            "type_symbols.rs should not own {label} test body: {test_name}"
        );
        assert!(
            module.contains(&format!("fn {test_name}")),
            "{label} tests should live in focused module: {test_name}"
        );
    }

    for test_name in [
        "check_program_with_symbols_validates_resolver_behavior_visibility",
        "check_program_with_symbols_validates_resolver_behavior_type_parameter_bounds",
        "check_program_with_symbols_validates_resolver_behavior_absent_type_metadata",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_type_behavior_metadata.rs should not own behavior metadata test: {test_name}"
        );
        assert!(
            behavior_metadata.contains(&format!("fn {test_name}")),
            "behavior metadata tests should live in focused module: {test_name}"
        );
    }

    for module_name in ["type_symbols", "behavior_symbols"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "resolver type/behavior metadata root should include focused module: {module_name}"
        );
    }
}

#[test]
fn typechecker_resolver_declaration_tests_live_in_focused_modules() {
    let root = read("src/typechecker/tests/resolver_declarations.rs");
    let symbols = read("src/typechecker/tests/resolver_declarations/symbols.rs");
    let imports = read("src/typechecker/tests/resolver_declarations/imports.rs");
    let import_extra_symbols =
        read("src/typechecker/tests/resolver_declarations/imports/extra_symbols.rs");
    let import_restored_bindings =
        read("src/typechecker/tests/resolver_declarations/imports/restored_bindings.rs");
    let import_stripped_metadata =
        read("src/typechecker/tests/resolver_declarations/imports/stripped_metadata.rs");
    let methods = read("src/typechecker/tests/resolver_declarations/methods.rs");

    for test_name in [
        "check_program_with_symbols_requires_resolver_declarations",
        "check_program_with_symbols_rejects_extra_resolver_declarations",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_declarations.rs should not own symbol declaration test: {test_name}"
        );
        assert!(
            symbols.contains(&format!("fn {test_name}")),
            "resolver declaration symbol tests should live in focused module: {test_name}"
        );
    }

    for test_name in [
        "check_program_with_symbols_rejects_extra_resolver_imports_when_ast_imports_are_present",
        "check_program_with_symbols_rejects_extra_resolver_modules_when_ast_imports_are_present",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_declarations.rs should not own resolver import test: {test_name}"
        );
        assert!(
            !imports.contains(&format!("fn {test_name}")),
            "imports.rs should not own extra import/module test body: {test_name}"
        );
        assert!(
            import_extra_symbols.contains(&format!("fn {test_name}")),
            "extra import/module tests should live in focused module: {test_name}"
        );
    }
    assert!(
        import_restored_bindings
            .contains("fn check_program_with_symbols_uses_resolver_import_bindings"),
        "restored_bindings.rs should cover resolver-backed import seeding"
    );
    for test_name in [
        "check_program_with_symbols_uses_resolver_import_bindings",
        "check_program_with_symbols_validates_stripped_resolver_import_sources",
        "check_program_with_symbols_validates_stripped_resolver_import_visibility",
        "check_program_with_symbols_requires_stripped_resolver_import_modules",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_declarations.rs should not own resolver import test: {test_name}"
        );
        assert!(
            !imports.contains(&format!("fn {test_name}")),
            "imports.rs should not own resolver import test body: {test_name}"
        );
    }
    for test_name in [
        "check_program_with_symbols_validates_stripped_resolver_import_sources",
        "check_program_with_symbols_validates_stripped_resolver_import_visibility",
        "check_program_with_symbols_requires_stripped_resolver_import_modules",
    ] {
        assert!(
            import_stripped_metadata.contains(&format!("fn {test_name}")),
            "stripped import metadata tests should live in focused module: {test_name}"
        );
    }
    for module_name in ["extra_symbols", "restored_bindings", "stripped_metadata"] {
        assert!(
            imports.contains(&format!("mod {module_name};")),
            "resolver declaration imports router should include focused module: {module_name}"
        );
    }

    for test_name in [
        "check_program_with_symbols_requires_resolver_method_receiver_type",
        "check_program_with_symbols_validates_resolver_method_signature",
        "check_program_with_symbols_validates_resolver_method_function_type_signature",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_declarations.rs should not own resolver method test: {test_name}"
        );
        assert!(
            methods.contains(&format!("fn {test_name}")),
            "resolver declaration method tests should live in focused module: {test_name}"
        );
    }

    for module_name in ["symbols", "imports", "methods"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "resolver declarations root should include focused module: {module_name}"
        );
    }
}
