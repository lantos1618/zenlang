use super::*;

#[test]
fn resolver_validation_docs_truth_stays_split_across_focused_modules() {
    let root = read("tests/docs_truth/repo_hygiene/typechecker_resolver_validation.rs");
    let focused_modules =
        read("tests/docs_truth/repo_hygiene/typechecker_resolver_validation/focused_modules.rs");
    let resolver_tests = read(
        "tests/docs_truth/repo_hygiene/typechecker_resolver_validation/focused_modules/resolver_tests.rs",
    );
    let behavior_impl_methods = read(
        "tests/docs_truth/repo_hygiene/typechecker_resolver_validation/focused_modules/behavior_impl_methods.rs",
    );
    let replay_and_entry = read(
        "tests/docs_truth/repo_hygiene/typechecker_resolver_validation/focused_modules/replay_and_entry.rs",
    );

    assert!(
        root.lines().count() < 260,
        "typechecker resolver-validation docs-truth guards should stay split across focused modules"
    );
    assert!(
        focused_modules.lines().count() < 60,
        "focused_modules.rs should route focused docs-truth guard modules"
    );
    for module_name in [
        "resolver_tests",
        "behavior_impl_methods",
        "replay_and_entry",
    ] {
        assert!(
            focused_modules.contains(&format!("mod {module_name};")),
            "focused_modules.rs should include focused guard module: {module_name}"
        );
    }
    for test_name in [
        "typechecker_resolver_type_behavior_metadata_tests_live_in_focused_modules",
        "typechecker_resolver_declaration_tests_live_in_focused_modules",
    ] {
        assert!(
            resolver_tests.contains(&format!("fn {test_name}")),
            "resolver test-module docs-truth guard should live in resolver_tests.rs: {test_name}"
        );
        assert!(
            !focused_modules.contains(&format!("fn {test_name}")),
            "focused_modules.rs should not own resolver test-module guard: {test_name}"
        );
    }
    for test_name in [
        "resolver_collection_behavior_impl_method_tests_live_in_focused_modules",
        "resolver_collection_behavior_impl_restored_generic_templates_live_in_focused_helper",
    ] {
        assert!(
            behavior_impl_methods.contains(&format!("fn {test_name}")),
            "behavior impl method docs-truth guard should live in behavior_impl_methods.rs: {test_name}"
        );
        assert!(
            !focused_modules.contains(&format!("fn {test_name}")),
            "focused_modules.rs should not own behavior impl method guard: {test_name}"
        );
    }
    for test_name in [
        "typechecker_resolver_entry_association_helpers_live_in_focused_helper",
        "typechecker_resolver_replay_association_tasks_live_in_focused_helper",
    ] {
        assert!(
            replay_and_entry.contains(&format!("fn {test_name}")),
            "entry/replay docs-truth guard should live in replay_and_entry.rs: {test_name}"
        );
        assert!(
            !focused_modules.contains(&format!("fn {test_name}")),
            "focused_modules.rs should not own entry/replay guard: {test_name}"
        );
    }
}
