use super::super::*;

#[test]
fn resolver_validation_docs_truth_stays_split_across_focused_modules() {
    let root = read("tests/docs_truth/repo_hygiene/typechecker_resolver_validation.rs");

    assert!(
        root.lines().count() < 260,
        "typechecker resolver-validation docs-truth guards should stay split across focused modules"
    );
}

#[test]
fn typechecker_test_split_guards_live_in_focused_module() {
    let root = read("tests/docs_truth/repo_hygiene/file_size/focused_tests.rs");
    let focused = read("tests/docs_truth/repo_hygiene/file_size/typechecker_test_splits.rs");

    for test_name in [
        "resolver_metadata_queue_selection_tests_live_in_focused_helper",
        "resolver_import_absence_tests_live_in_focused_helper",
        "resolver_required_stale_diagnostic_tests_live_in_focused_helper",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "focused_tests.rs should not own typechecker test split guard: {test_name}"
        );
        assert!(
            focused.contains(&format!("fn {test_name}")),
            "typechecker test split guard should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 120,
        "focused_tests.rs should stay focused on top-level file-size guard grouping"
    );
}

#[test]
fn typechecker_semantic_guard_tests_live_in_focused_module() {
    let root = read("tests/docs_truth/repo_hygiene/file_size/typechecker_test_splits.rs");
    let semantic = read("tests/docs_truth/repo_hygiene/file_size/typechecker_semantic_splits.rs");
    let includes = read("tests/docs_truth/repo_hygiene/file_size.rs");

    for test_name in [
        "declaration_validation_precollection_tasks_live_in_focused_helper",
        "struct_literal_default_tests_live_in_focused_helper",
        "generic_behavior_impl_type_arg_tests_live_in_focused_helper",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "typechecker_test_splits.rs should not own semantic guard test: {test_name}"
        );
        assert!(
            semantic.contains(&format!("fn {test_name}")),
            "semantic typechecker guard test should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 180,
        "typechecker_test_splits.rs should stay focused on resolver metadata guard splits"
    );
    assert!(
        includes.contains("mod typechecker_semantic_splits;"),
        "file_size.rs should include the focused typechecker semantic split guards"
    );
}
