use super::*;

#[test]
fn resolver_collection_docs_truth_guards_stay_split_by_surface() {
    let root = read(
        "tests/docs_truth/repo_hygiene/file_size/focused_tests/typechecker/resolver_collection.rs",
    );
    let generic_templates = read(
        "tests/docs_truth/repo_hygiene/file_size/focused_tests/typechecker/resolver_collection/generic_templates.rs",
    );
    let metadata_helpers = read(
        "tests/docs_truth/repo_hygiene/file_size/focused_tests/typechecker/resolver_collection/metadata_helpers.rs",
    );
    let metadata_restoration = read(
        "tests/docs_truth/repo_hygiene/file_size/focused_tests/typechecker/resolver_collection/metadata_restoration.rs",
    );
    let metadata_requirements = read(
        "tests/docs_truth/repo_hygiene/file_size/focused_tests/typechecker/resolver_collection/metadata_requirements.rs",
    );

    assert!(
        root.lines().count() < 80,
        "resolver_collection.rs should route focused docs-truth guard surfaces"
    );
    for module in [
        "mod generic_templates;",
        "mod metadata_helpers;",
        "mod metadata_requirements;",
        "mod metadata_restoration;",
    ] {
        assert!(
            root.contains(module),
            "resolver_collection.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains(
            "fn resolver_collection_generic_function_template_tests_stay_split_by_responsibility"
        ),
        "generic template guards should live in focused child modules"
    );

    assert!(
        generic_templates.contains(
            "fn resolver_collection_generic_function_template_tests_stay_split_by_responsibility"
        ) && generic_templates.contains(
            "fn resolver_collection_generic_method_template_tests_stay_split_by_responsibility"
        ),
        "generic template guards should live in generic_templates.rs"
    );
    assert!(
        metadata_helpers
            .contains("fn resolver_collection_type_metadata_tests_stay_split_by_responsibility")
            && metadata_helpers
                .contains("fn resolver_metadata_queue_selection_tests_live_in_focused_helper")
            && metadata_helpers.contains(
                "fn resolver_metadata_impl_and_method_helper_tests_stay_split_by_responsibility"
            ),
        "metadata helper guards should live in metadata_helpers.rs"
    );
    assert!(
        metadata_restoration
            .contains("fn resolver_metadata_restoration_tests_stay_split_by_responsibility"),
        "metadata restoration guard should live in metadata_restoration.rs"
    );
    assert!(
        metadata_requirements
            .contains("fn resolver_metadata_requirement_tests_stay_split_by_responsibility"),
        "metadata requirement guard should live in metadata_requirements.rs"
    );
}
