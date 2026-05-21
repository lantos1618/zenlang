use super::super::*;

#[test]
fn resolver_behavior_ref_validation_message_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/resolver_validation/behavior_refs.rs");
    let messages =
        read("src/typechecker/tests/resolver_validation/behavior_refs/validation_messages.rs");

    for test_name in [
        "behavior_ref_validation_maps_role_and_check_diagnostics",
        "behavior_ref_validation_separates_role_labels_from_check_codes",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "behavior_refs.rs should not own validation message test: {test_name}"
        );
        assert!(
            messages.contains(&format!("fn {test_name}")),
            "validation message test should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 180,
        "behavior_refs.rs should stay focused on behavior ref metadata selection"
    );
    assert!(
        root.contains("mod validation_messages;"),
        "behavior_refs.rs should include focused validation message tests"
    );
}

#[test]
fn resolver_imported_type_dependency_seeding_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let imports = read("src/typechecker/resolver_validation/imports_dependencies.rs");
    let type_dependencies =
        read("src/typechecker/resolver_validation/imports_type_dependencies.rs");

    assert!(
        !imports.contains("fn seed_imported_type_dependency"),
        "imports_dependencies.rs should not own imported type dependency seeding"
    );
    assert!(
        type_dependencies.contains("fn seed_imported_type_dependency"),
        "imports_type_dependencies.rs should own imported type dependency seeding"
    );
    assert!(
        imports.lines().count() < 200,
        "imports_dependencies.rs should stay focused on callable/template/import seeding"
    );
    assert!(
        root.contains("include!(\"resolver_validation/imports_type_dependencies.rs\");"),
        "resolver validation should include focused imported type dependency seeding"
    );
}

#[test]
fn resolver_imported_behavior_dependency_seeding_stays_split_by_responsibility() {
    let root = read("src/typechecker/resolver_validation/imports_behavior_dependencies.rs");
    let extends =
        read("src/typechecker/resolver_validation/imports_behavior_dependencies/extends.rs");
    let impls = read("src/typechecker/resolver_validation/imports_behavior_dependencies/impls.rs");

    assert!(
        root.lines().count() < 80,
        "imports_behavior_dependencies.rs should only include focused behavior dependency helpers"
    );
    assert!(
        root.contains("include!(\"imports_behavior_dependencies/extends.rs\");"),
        "behavior dependency root should include imported behavior inheritance helper"
    );
    assert!(
        root.contains("include!(\"imports_behavior_dependencies/impls.rs\");"),
        "behavior dependency root should include imported behavior impl helper"
    );
    for helper in [
        "seed_behavior_extends_for_imported_behavior",
        "seed_behavior_extends_for_imported_behavior_inner",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "behavior dependency root should not own imported inheritance helper: {helper}"
        );
        assert!(
            extends.contains(&format!("fn {helper}")),
            "imported behavior inheritance helper should live in extends.rs: {helper}"
        );
    }
    for helper in [
        "seed_behavior_impls_for_imported_type",
        "imported_behavior_impl_is_public",
        "seed_behavior_decl_for_imported_impl",
        "seed_behavior_decl_for_imported_impl_from_imports",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "behavior dependency root should not own imported impl helper: {helper}"
        );
        assert!(
            impls.contains(&format!("fn {helper}")),
            "imported behavior impl helper should live in impls.rs: {helper}"
        );
    }
}
