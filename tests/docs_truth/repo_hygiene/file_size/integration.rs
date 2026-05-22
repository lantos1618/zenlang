use super::super::*;

mod import_visibility_dependencies;
mod multi_file_fixtures;
mod single_file_fixtures;

#[test]
fn integration_file_size_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/file_size/integration.rs");
    let import_visibility = read(
        "tests/docs_truth/repo_hygiene/file_size/integration/import_visibility_dependencies.rs",
    );
    let multi_file =
        read("tests/docs_truth/repo_hygiene/file_size/integration/multi_file_fixtures.rs");
    let single_file =
        read("tests/docs_truth/repo_hygiene/file_size/integration/single_file_fixtures.rs");

    assert!(
        root.lines().count() < 80,
        "integration.rs should route focused integration file-size guard modules"
    );
    for module_name in [
        "import_visibility_dependencies",
        "multi_file_fixtures",
        "single_file_fixtures",
    ] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "integration.rs should include focused guard module: {module_name}"
        );
    }
    assert!(
        import_visibility
            .contains("fn import_visibility_dependency_tests_stay_split_by_dependency_shape"),
        "import visibility dependency guards should live in import_visibility_dependencies.rs"
    );
    assert!(
        multi_file.contains("fn multi_file_fixture_tests_stay_split_by_fixture_family"),
        "multi-file fixture guards should live in multi_file_fixtures.rs"
    );
    assert!(
        single_file.contains("fn single_file_fixture_tests_stay_split_by_fixture_family"),
        "single-file fixture guards should live in single_file_fixtures.rs"
    );
}
