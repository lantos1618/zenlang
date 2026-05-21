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

#[test]
fn resolver_collection_behavior_impl_restored_generic_templates_live_in_focused_helper() {
    let root = read(
        "src/typechecker/tests/resolver_collection/behavior_impl_methods/restored_signatures.rs",
    );
    let generic_templates = read(
        "src/typechecker/tests/resolver_collection/behavior_impl_methods/restored_signatures/generic_templates.rs",
    );

    for test_name in [
        "collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata",
        "collect_declarations_with_symbols_clears_stale_behavior_impl_generic_method_template_after_key_restore",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "restored_signatures.rs should not own generic restored-template test: {test_name}"
        );
        assert!(
            generic_templates.contains(&format!("fn {test_name}")),
            "generic restored-template test should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 180,
        "restored_signatures.rs should stay focused on non-generic signature restoration"
    );
    assert!(
        root.contains("mod generic_templates;"),
        "restored_signatures.rs should include focused generic-template restoration tests"
    );
}

#[test]
fn typechecker_resolver_entry_association_helpers_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let entry = read("src/typechecker/resolver_validation/entry_symbols.rs");
    let associations = read("src/typechecker/resolver_validation/entry_associations.rs");

    for helper in [
        "validate_resolver_impl_block_entry",
        "validate_resolver_requires_entry",
        "validate_resolver_behavior_extends_entry",
    ] {
        assert!(
            !entry.contains(&format!("fn {helper}")),
            "resolver entry traversal should not own behavior-association helper: {helper}"
        );
        assert!(
            entry.contains(&format!("self.{helper}(")),
            "resolver entry traversal should delegate behavior-association work through {helper}"
        );
        assert!(
            associations.contains(&format!("fn {helper}")),
            "resolver behavior-association entry helper should live in focused helper: {helper}"
        );
    }

    assert!(
        entry.lines().count() < 220,
        "resolver entry traversal should stay focused on declaration dispatch"
    );
    assert!(
        root.contains("include!(\"resolver_validation/entry_associations.rs\");"),
        "resolver validation should include focused entry behavior-association helpers"
    );
}

#[test]
fn typechecker_resolver_replay_association_tasks_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let replay = read("src/typechecker/resolver_validation/replay_tasks.rs");
    let associations = read("src/typechecker/resolver_validation/replay_task_associations.rs");

    for helper in [
        "collect_resolver_behavior_association_list_tasks_from_declaration_tasks",
        "push_resolver_type_behavior_association_list_task",
        "push_resolver_behavior_parent_list_task",
    ] {
        assert!(
            !replay.contains(&format!("fn {helper}")),
            "resolver replay task root should not own behavior-association replay helper: {helper}"
        );
        assert!(
            associations.contains(&format!("fn {helper}")),
            "behavior-association replay helper should live in focused helper: {helper}"
        );
    }

    assert!(
        replay.lines().count() < 210,
        "resolver replay task root should stay focused on declaration replay collection"
    );
    assert!(
        root.contains("include!(\"resolver_validation/replay_task_associations.rs\");"),
        "resolver validation should include focused behavior-association replay helpers"
    );
}
