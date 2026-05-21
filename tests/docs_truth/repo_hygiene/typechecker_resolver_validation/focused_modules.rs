use super::*;

#[test]
fn typechecker_resolver_type_behavior_metadata_tests_live_in_focused_modules() {
    let root = read("src/typechecker/tests/resolver_type_behavior_metadata.rs");
    let type_metadata =
        read("src/typechecker/tests/resolver_type_behavior_metadata/type_symbols.rs");
    let behavior_metadata =
        read("src/typechecker/tests/resolver_type_behavior_metadata/behavior_symbols.rs");

    for test_name in [
        "check_program_with_symbols_validates_resolver_type_parameter_counts",
        "check_program_with_symbols_validates_resolver_type_parameter_names",
        "check_program_with_symbols_validates_resolver_type_visibility",
        "check_program_with_symbols_validates_resolver_type_parameter_bounds",
        "check_program_with_symbols_validates_resolver_type_like_absent_value_metadata",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "resolver_type_behavior_metadata.rs should not own type metadata test: {test_name}"
        );
        assert!(
            type_metadata.contains(&format!("fn {test_name}")),
            "type metadata tests should live in focused module: {test_name}"
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
            imports.contains(&format!("fn {test_name}")),
            "resolver declaration import tests should live in focused module: {test_name}"
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

#[test]
fn resolver_collection_behavior_impl_method_tests_live_in_focused_modules() {
    let root = read("src/typechecker/tests/resolver_collection/behavior_impl_methods/mod.rs");

    assert!(
        root.lines().count() < 260,
        "resolver collection behavior impl method tests should live in focused modules"
    );
}
