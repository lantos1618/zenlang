use super::super::*;

#[test]
fn examples_index_uses_canonical_tutorial_and_project_paths() {
    let examples = read("examples/README.md");

    for required in [
        "docs/learn_zen_in_y_minutes.md",
        "examples/01_hello_world.zen",
        "examples/02_variables_and_types.zen",
        "examples/03_pattern_matching.zen",
        "examples/04_structs_and_methods.zen",
        "examples/05_loops.zen",
        "examples/06_error_handling.zen",
        "examples/project/main.zen",
        "examples/project/test.zen",
    ] {
        assert!(
            examples.contains(required),
            "examples/README.md is missing canonical path: {required}"
        );
    }

    for stale_path in [
        "examples/hello_world.zen",
        "examples/variables_and_types.zen",
        "examples/pattern_matching.zen",
        "examples/structs_and_methods.zen",
        "examples/loops_and_closures.zen",
        "examples/error_handling.zen",
        "examples/demo_project",
    ] {
        assert!(
            !examples.contains(stale_path),
            "examples/README.md still references redundant path: {stale_path}"
        );
    }
}

#[test]
fn examples_directory_contains_only_canonical_public_examples() {
    let examples_dir = repo_root().join("examples");
    let mut actual = std::fs::read_dir(&examples_dir)
        .expect("examples directory should exist")
        .map(|entry| {
            entry
                .expect("examples entry should be readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    actual.sort();

    assert_eq!(
        actual,
        [
            "01_hello_world.zen",
            "02_variables_and_types.zen",
            "03_pattern_matching.zen",
            "04_structs_and_methods.zen",
            "05_loops.zen",
            "06_error_handling.zen",
            "README.md",
            "project",
        ],
        "examples/ should contain only canonical public examples"
    );
}

#[test]
fn stale_generated_tooling_directories_are_removed() {
    for path in [
        ".claude",
        "scripts",
        "examples/demo_project",
        "main",
        "main.c",
    ] {
        let absolute_path = repo_root().join(path);
        assert!(
            !absolute_path.exists(),
            "{path} should not exist; stale generated tooling/examples/build outputs should stay out of the repo"
        );
    }
}
