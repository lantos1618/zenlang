use std::process::Output;

use super::support::{
    assert_path_exists, assert_test_binary_and_output, assert_zen_success, build_graph_source,
    run_zen_in, write_file, MAIN_ZERO, TEST_ARGS,
};

#[test]
fn test_command_build_zen_runs_test_targets() {
    let (tmp, output) = run_test_command(
        &[r#"    b.add(Test { name: "unit", root: "test.zen" })"#],
        &["test.zen"],
    );

    assert_test_binary_and_output(&tmp, &output, "unit");
}

#[test]
fn test_command_build_zen_runs_multiple_test_targets() {
    let (tmp, output) = run_test_command(
        &[
            r#"    b.add(Test { name: "unit", root: "unit.zen" })"#,
            r#"    b.add(Test { name: "integration", root: "integration.zen" })"#,
        ],
        &["unit.zen", "integration.zen"],
    );

    for name in ["unit", "integration"] {
        assert_test_binary_and_output(&tmp, &output, name);
    }
}

#[test]
fn test_command_build_zen_runs_test_dependencies_first() {
    let (tmp, output) = run_test_command(
        &[r#"    b.add(Test {
        name: "integration",
        root: "integration.zen",
        dependencies: ["unit"],
    })
    b.add(Test { name: "unit", root: "unit.zen" })"#],
        &["unit.zen", "integration.zen"],
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let unit_pass = stdout
        .find("test unit passed")
        .unwrap_or_else(|| panic!("expected unit test pass output, stdout={stdout}"));
    let integration_pass = stdout
        .find("test integration passed")
        .unwrap_or_else(|| panic!("expected integration test pass output, stdout={stdout}"));
    assert!(
        unit_pass < integration_pass,
        "expected dependency test target to run before dependent target, stdout={stdout}"
    );

    for name in ["unit", "integration"] {
        assert_path_exists(tmp.path().join("build").join("tests").join(name));
    }
}

fn run_test_command(targets: &[&str], source_paths: &[&str]) -> (tempfile::TempDir, Output) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, "build.zen", &build_graph_source(targets));

    for path in source_paths {
        write_file(&tmp, path, MAIN_ZERO);
    }

    let output = run_zen_in(&tmp, TEST_ARGS);
    assert_zen_success(TEST_ARGS, &output);
    (tmp, output)
}
