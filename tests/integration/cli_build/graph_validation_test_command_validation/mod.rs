use super::support::{
    assert_no_build_dir, assert_test_binary_and_output, assert_zen_failure_contains,
    assert_zen_success, run_build_zen_command, LIBRARY_SOURCE, MAIN_ZERO, TEST_ARGS,
};
#[test]
fn test_command_build_zen_rejects_graph_without_test_targets() {
    let (tmp, output) = run_test_command(
        &[r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#],
        &[("app.zen", MAIN_ZERO)],
    );
    assert_zen_failure_contains(
        TEST_ARGS,
        &output,
        "build graph test execution requires at least one test target",
    );
    assert_no_build_dir(tmp.path(), "test command");
}

#[test]
fn test_command_build_zen_accepts_library_dependencies() {
    assert_test_command_succeeds(
        &[
            r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
            r#"    b.add(Test { name: "unit", root: "test.zen", dependencies: ["core"] })"#,
        ],
        &[("lib.zen", LIBRARY_SOURCE), ("test.zen", MAIN_ZERO)],
    );
}

#[test]
fn test_command_build_zen_ignores_unrelated_gated_executable_source_errors() {
    assert_test_command_succeeds(
        &[
            r#"    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })"#,
            r#"    b.add(Test { name: "unit", root: "test.zen" })"#,
        ],
        &[("test.zen", MAIN_ZERO)],
    );
}

fn assert_test_command_succeeds(targets: &[&str], files: &[(&str, &str)]) {
    let (tmp, output) = run_test_command(targets, files);
    assert_zen_success(TEST_ARGS, &output);
    assert_test_binary_and_output(&tmp, &output, "unit");
}

fn run_test_command(
    targets: &[&str],
    files: &[(&str, &str)],
) -> (tempfile::TempDir, std::process::Output) {
    run_build_zen_command(TEST_ARGS, targets, files)
}
